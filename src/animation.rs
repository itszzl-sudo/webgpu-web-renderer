//! 动画引擎
//! 
//! 支持 CSS @keyframes 动画的时间线管理、插值和状态计算。

use crate::css::parser::Keyframe;

/// 动画填充模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FillMode {
    None,
    Forwards,
    Backwards,
    Both,
}

/// 动画方向
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}

/// 单个元素的动画状态
#[derive(Debug, Clone)]
pub struct AnimState {
    /// 动画名称 (对应 @keyframes name)
    pub name: String,
    /// 已流逝时间 (秒)
    pub elapsed: f32,
    /// 持续时间 (秒)
    pub duration: f32,
    /// 延迟时间 (秒)
    pub delay: f32,
    /// 迭代次数
    pub iteration_count: f32,
    /// 填充模式
    pub fill_mode: FillMode,
    /// 方向
    pub direction: Direction,
    /// 是否正在运行
    pub running: bool,
}

impl AnimState {
    pub fn new(name: &str) -> Self {
        AnimState {
            name: name.to_string(),
            elapsed: 0.0,
            duration: 0.3,
            delay: 0.0,
            iteration_count: 1.0,
            fill_mode: FillMode::None,
            direction: Direction::Normal,
            running: true,
        }
    }

    /// 推进时间
    pub fn advance(&mut self, dt: f32) {
        if !self.running {
            return;
        }
        self.elapsed += dt;
        // 检查是否已完成所有迭代
        let total_duration = self.delay + self.duration * self.iteration_count;
        if self.elapsed >= total_duration && self.iteration_count > 0.0 {
            self.running = false;
            self.elapsed = total_duration;
        }
    }

    /// 计算当前动画进度 (0.0 ~ 1.0), 已考虑 delay 和 direction
    pub fn progress(&self) -> f32 {
        if !self.running && self.elapsed >= self.delay + self.duration {
            if self.fill_mode == FillMode::Forwards || self.fill_mode == FillMode::Both {
                return match self.direction {
                    Direction::Reverse | Direction::AlternateReverse => 0.0,
                    _ => 1.0,
                };
            }
            return 0.0; // fill-mode: none
        }

        // delay 期间无动画
        if self.elapsed < self.delay {
            if self.fill_mode == FillMode::Backwards || self.fill_mode == FillMode::Both {
                return match self.direction {
                    Direction::Reverse | Direction::AlternateReverse => 1.0,
                    _ => 0.0,
                };
            }
            return 0.0;
        }

        // 计算当前迭代内的进度
        let cycle_time = self.elapsed - self.delay;
        let cycle_duration = self.duration;
        let iteration = (cycle_time / cycle_duration) as u32;
        let t = (cycle_time % cycle_duration) / cycle_duration;

        // 处理 direction
        match self.direction {
            Direction::Normal | Direction::Reverse if iteration % 2 == 0 => {
                if self.direction == Direction::Reverse { 1.0 - t } else { t }
            }
            Direction::Alternate => {
                if iteration % 2 == 0 { t } else { 1.0 - t }
            }
            Direction::AlternateReverse => {
                if iteration % 2 == 0 { 1.0 - t } else { t }
            }
            _ => t,
        }
    }

    /// 重置动画
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.running = true;
    }
}

/// 在两个关键帧之间插值
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// 解析一个 CSS 属性值，提取数值部分用于插值
fn extract_numeric(value: &str) -> Option<f32> {
    let v = value.trim();
    if let Some(px) = v.strip_suffix("px") {
        px.trim().parse::<f32>().ok()
    } else if let Some(em) = v.strip_suffix("em") {
        em.trim().parse::<f32>().ok().map(|x| x * 16.0)
    } else if let Some(pct) = v.strip_suffix('%') {
        pct.trim().parse::<f32>().ok()
    } else {
        v.parse::<f32>().ok()
    }
}

/// 将有单位的值格式化为字符串 (px)
fn format_numeric(val: f32, original: &str) -> String {
    if original.ends_with('%') {
        format!("{}%", val)
    } else if original.ends_with("em") {
        format!("{}em", val / 16.0)
    } else {
        format!("{}px", val)
    }
}

/// 根据动画进度从 keyframes 中插值计算属性值
/// 
/// 对可插值的数值属性 (width, height, opacity, left, top 等) 进行线性插值。
/// 对不可插值的属性 (display, visibility 等) 返回最接近的关键帧值。
pub fn interpolate_keyframes(keyframes: &[Keyframe], property: &str, progress: f32) -> Option<String> {
    if keyframes.is_empty() {
        return None;
    }

    // 收集所有关键帧中此属性的值
    let mut values: Vec<(f32, String)> = Vec::new();
    for kf in keyframes {
        for decl in &kf.declarations {
            if decl.property == property {
                values.push((kf.selector, decl.value.clone()));
            }
        }
    }

    if values.is_empty() {
        return None;
    }

    // 找前后关键帧
    if progress <= values[0].0 {
        return Some(values[0].1.clone());
    }
    if progress >= values[values.len() - 1].0 {
        return Some(values[values.len() - 1].1.clone());
    }

    for i in 0..values.len() - 1 {
        let (t0, ref v0) = values[i];
        let (t1, ref v1) = values[i + 1];
        if progress >= t0 && progress <= t1 {
            let local_t = if (t1 - t0).abs() < 0.0001 {
                0.5
            } else {
                (progress - t0) / (t1 - t0)
            };

            // 尝试数值插值
            if let (Some(n0), Some(n1)) = (extract_numeric(v0), extract_numeric(v1)) {
                let interpolated = lerp(n0, n1, local_t);
                return Some(format_numeric(interpolated, v0));
            }

            // 否则返回起始值 (不可插值的属性，如颜色名、display 等)
            return Some(if local_t < 0.5 { v0.clone() } else { v1.clone() });
        }
    }

    None
}