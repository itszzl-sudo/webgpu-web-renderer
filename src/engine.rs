use crate::bridge::{WebNativeBridge, LayoutRect, LayoutNode, Color};
use crate::dom::tree::DomTree;
use crate::dom::parser::HtmlParser;
use crate::css::parser::StyleSheet;
use crate::css::matcher::StyleMatcher;
use crate::layout::{LayoutItem, LayoutEnv, LayoutConverter, LayoutCompute, CpuLayoutCompute};
use crate::renderer::pipeline::RenderPipelineWrapper;
use crate::animation::{AnimState, interpolate_keyframes};
use crate::network::HttpResponse;
use image::{RgbaImage, ImageFormat};
use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;

// Type aliases for cleaner code
type ClickHandler = Rc<RefCell<dyn FnMut(f32, f32)>>;
type FormHandler = Rc<RefCell<dyn FnMut(HashMap<String, String>)>>;
type WindowOpenHandler = Rc<RefCell<dyn FnMut(&str) -> bool>>;

/// 脏标记位掩码 — 跟踪哪些状态需要重新计算
const DIRTY_NONE: u8 = 0;
const DIRTY_HTML: u8 = 1 << 0;        // HTML 内容已变
const DIRTY_CSS: u8 = 1 << 1;         // CSS 样式表已变
const DIRTY_INLINE_STYLE: u8 = 1 << 2; // 内联样式已变
const DIRTY_VIEWPORT: u8 = 1 << 3;    // 视口尺寸已变
const DIRTY_LAYOUT: u8 = 1 << 4;      // 需要重新布局 (CSS/HTML/viewport 等任意变化)

/// HTML 标签默认样式表 (User Agent Stylesheet)
const UA_STYLESHEET: &str = r#"
/* ── 块级元素 ── */
div, p, h1, h2, h3, h4, h5, h6, ul, ol, li,
section, header, footer, nav, article, aside, main,
figure, figcaption, dl, dt, dd, blockquote, pre,
form, fieldset, details, summary { display: block; }

/* ── 内联元素 ── */
span, a, strong, em, b, i, u, small, code, label,
abbr, cite, dfn, kbd, mark, q, s, sub, sup, time, var { display: inline; }

/* ── 内联块级元素 ── */
img, input, button, textarea, select { display: inline-block; }

/* ── 标题 ── */
h1 { font-size: 2em; margin: 0.67em 0; font-weight: bold; }
h2 { font-size: 1.5em; margin: 0.75em 0; font-weight: bold; }
h3 { font-size: 1.17em; margin: 0.83em 0; font-weight: bold; }
h4 { font-size: 1em; margin: 1.12em 0; font-weight: bold; }
h5 { font-size: 0.83em; margin: 1.5em 0; font-weight: bold; }
h6 { font-size: 0.67em; margin: 1.67em 0; font-weight: bold; }

/* ── 段落 ── */
p { margin: 1em 0; }

/* ── 列表 ── */
ul, ol { padding-left: 40px; }
li { display: list-item; }

/* ── 链接 ── */
a { color: blue; text-decoration: underline; cursor: pointer; }

/* ── 行内代码 / 预格式化 ── */
pre { font-family: monospace; white-space: pre; }
code { font-family: monospace; }

/* ── 页面主体 ── */
body { margin: 8px; display: block; }

/* ── 水平分割线 ── */
hr { display: block; margin: 0.5em auto; border-style: inset; border-width: 1px; }

/* ── 表单控件 ── */
input { padding: 1px; border-width: 1px; }
button { padding: 1px 6px; border-width: 1px; cursor: pointer; }
textarea { padding: 2px; border-width: 1px; }
select { border-width: 1px; }
label { cursor: default; }

/* ── 表格 ── */
table { display: table; }
tr { display: table-row; }
td, th { display: table-cell; padding: 1px; }
th { font-weight: bold; text-align: center; }

/* ── 图片 ── */
img { max-width: 100%; }

/* ── 引用 ── */
blockquote { margin: 1em 40px; }
figcaption { display: block; }
"#;

pub struct Engine {
    width: u32,
    height: u32,
    device: Option<Arc<wgpu::Device>>,
    queue: Option<Arc<wgpu::Queue>>,
    dom_tree: DomTree,
    stylesheet: StyleSheet,
    style_matcher: Option<StyleMatcher>,
    layout_items: Vec<LayoutItem>,
    render_pipeline: Option<RenderPipelineWrapper>,
    layout_compute: Option<LayoutCompute>,
    click_handlers: HashMap<usize, Vec<ClickHandler>>,
    form_handlers: HashMap<String, FormHandler>,
    window_open_handlers: Vec<WindowOpenHandler>,
    current_url: String,
    // ── 动画字段 ──
    animation_time: f32,
    animation_states: Vec<AnimState>,
    // ── 增量更新字段 ──
    dirty_flags: u8,               // 脏标记位掩码
    pending_html: Option<String>,  // 待提交的 HTML
    pending_css: Option<String>,   // 待提交的 CSS
    pending_viewport: Option<(u32, u32)>, // 待提交的视口
}

impl Engine {
    /// 同步初始化 WebGPU
    fn init_webgpu_sync(&mut self) -> Result<(), String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // 使用 poll 来同步请求 adapter
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        })).ok_or("Failed to find an appropriate adapter")?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("WebGPU Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        )).map_err(|e| format!("Failed to create device: {}", e))?;

        self.device = Some(Arc::new(device));
        self.queue = Some(Arc::new(queue));

        log::info!("WebGPU initialized successfully");
        Ok(())
    }

    /// 更新布局
    fn update_layout(&mut self) {
        if let Some(ref matcher) = self.style_matcher {
            let env = LayoutEnv::new(self.width as f32, self.height as f32);

            // 创建布局转换器 (需要克隆 matcher)
            let converter = LayoutConverter::new((*matcher).clone(), env.clone());

            // 转换 DOM 节点到 LayoutItem
            match converter.convert_dom(&self.dom_tree) {
                Ok(items) => {
                    let item_count = items.len();
                    self.layout_items = items;

                    // 尝试 GPU 布局计算，失败时回退到 CPU
                    let gpu_used = if let Some(ref lc) = self.layout_compute {
                        if let Err(e) = lc.update_items(&self.layout_items) {
                            log::warn!("GPU items upload failed (CPU fallback): {}", e);
                            false
                        } else if let Err(e) = lc.update_env(&env) {
                            log::warn!("GPU env upload failed (CPU fallback): {}", e);
                            false
                        } else if let Err(e) = lc.compute(item_count as u32) {
                            log::warn!("GPU compute failed (CPU fallback): {}", e);
                            false
                        } else if let Err(e) = lc.read_results(&mut self.layout_items) {
                            log::warn!("GPU readback failed (CPU fallback): {}", e);
                            false
                        } else {
                            log::info!("GPU layout compute: {} items", item_count);
                            true
                        }
                    } else {
                        false
                    };

                    if !gpu_used {
                        // CPU 布局计算回退
                        CpuLayoutCompute::compute(&mut self.layout_items, &env);
                        log::info!("CPU layout compute: {} items", item_count);
                    }

                    // 计算包含块（为 absolute 元素调整相对于最近定位祖先的位置）
                    self.compute_containing_blocks();

                    // Flexbox 布局计算
                    self.compute_flex_layout();

                    // Float 浮动布局
                    self.compute_float_layout();

                    // Inline 格式化布局
                    self.compute_inline_layout();

                    // 计算裁剪矩形（基于父容器 overflow 设置）
                    self.compute_clip_rects();

                    log::info!("Layout finished: {} items", self.layout_items.len());
                }
                Err(e) => {
                    log::error!("Layout conversion failed: {}", e);
                }
            }
        }
    }

    /// 计算包含块 — 为 absolute 元素调整相对于最近定位祖先的位置
    fn compute_containing_blocks(&mut self) {
        // 建立 dom_id → LayoutItem 索引
        let idx_map: std::collections::HashMap<usize, usize> = self.layout_items.iter()
            .enumerate()
            .map(|(i, it)| (it.dom_id as usize, i))
            .collect();

        let dom_ids: Vec<usize> = self.layout_items.iter().map(|it| it.dom_id as usize).collect();

        for &dom_id in &dom_ids {
            let idx = match idx_map.get(&dom_id) {
                Some(&i) => i,
                None => continue,
            };
            let item = &self.layout_items[idx];
            if item.flow_type != 1 {
                // 仅对 absolute/fixed 元素处理包含块
                continue;
            }

            // 向上遍历 DOM 祖先，找最近定位祖先 (position ≠ static)
            let mut current = dom_id;
            let mut container_pos = None;
            loop {
                let parent_id = match self.dom_tree.get_node(current) {
                    Some(n) => n.parent,
                    None => break,
                };
                let pid = match parent_id {
                    Some(p) => p,
                    None => break,
                };
                if let Some(&pidx) = idx_map.get(&pid) {
                    let parent = &self.layout_items[pidx];
                    // 定位祖先：relative, absolute, fixed, sticky
                    if parent.flow_type != 0 && parent.flow_type != 2 && parent.flow_type != 3 {
                        // 包含块 = 定位祖先的内容区域 (padding box)
                        container_pos = Some((
                            parent.pos[0] + parent.padding[3],  // x + padding-left
                            parent.pos[1] + parent.padding[0],  // y + padding-top
                        ));
                        break;
                    }
                }
                current = pid;
            }

            // 应用包含块偏移
            if let Some((cx, cy)) = container_pos {
                self.layout_items[idx].pos[0] += cx;
                self.layout_items[idx].pos[1] += cy;
            }
        }
    }

    /// Flexbox 布局计算 — 找出所有 flex 容器并对其子元素排布
    fn compute_flex_layout(&mut self) {
        // 收集 flex 容器的数据 (先读后写，避免借用冲突)
        struct FlexContainer {
            child_indices: Vec<usize>,
            flex_dir: u32,
            justify: u32,
            align: u32,
            gap: f32,
            available_main: f32,
            container_pos: [f32; 2],
            container_padding: [f32; 4],
            container_cross: f32,
        }

        let mut flex_containers: Vec<FlexContainer> = Vec::new();

        for container_idx in 0..self.layout_items.len() {
            let container = &self.layout_items[container_idx];
            if container.flow_type != 2 || container.is_valid == 0 || container.is_hide == 1 {
                continue;
            }
            let container_dom_id = container.dom_id as usize;

            let child_indices: Vec<usize> = self.layout_items.iter()
                .enumerate()
                .filter(|(_, it)| {
                    if it.is_valid == 0 || it.is_hide == 1 { return false; }
                    self.dom_tree.get_node(it.dom_id as usize)
                        .and_then(|n| n.parent)
                        .map_or(false, |p| p == container_dom_id)
                })
                .map(|(i, _)| i)
                .collect();

            if child_indices.is_empty() {
                continue;
            }

            let is_row = container.flex_direction == 0 || container.flex_direction == 1;
            let available_main = if is_row { container.size[0] } else { container.size[1] };

            flex_containers.push(FlexContainer {
                child_indices,
                flex_dir: container.flex_direction,
                justify: container.justify_content,
                align: container.align_items,
                gap: container.gap,
                available_main,
                container_pos: container.pos,
                container_padding: container.padding,
                container_cross: if is_row { container.size[1] } else { container.size[0] },
            });
        }

        // 执行 flex 布局
        for fc in &flex_containers {
            let is_row = fc.flex_dir == 0 || fc.flex_dir == 1;
            let is_reverse = fc.flex_dir == 1 || fc.flex_dir == 3;

            // 按 order 排序
            let mut sorted: Vec<usize> = fc.child_indices.clone();
            sorted.sort_by_key(|&i| self.layout_items[i].order);

            // 计算总 flex-grow 和基础尺寸
            let mut total_grow = 0.0f32;
            let mut total_basis = 0.0f32;
            for &idx in &sorted {
                total_grow += self.layout_items[idx].weight;
                let basis = self.layout_items[idx].flex_basis.max(0.0);
                let item_main = if is_row {
                    self.layout_items[idx].size[0].max(basis)
                } else {
                    self.layout_items[idx].size[1].max(basis)
                };
                total_basis += item_main;
            }

            // 计算剩余空间
            let item_count = sorted.len() as f32;
            let gaps = fc.gap * (item_count - 1.0).max(0.0);
            let remaining = fc.available_main - total_basis - gaps;

            // 分配空间
            let mut main_offset = 0.0;
            for (_pos, &idx) in sorted.iter().enumerate() {
                let basis = self.layout_items[idx].flex_basis.max(0.0);
                let base_main = if is_row {
                    self.layout_items[idx].size[0].max(basis)
                } else {
                    self.layout_items[idx].size[1].max(basis)
                };

                let extra = if total_grow > 0.0 && remaining > 0.0 {
                    (self.layout_items[idx].weight / total_grow) * remaining
                } else {
                    0.0
                };

                let final_main = base_main + extra;
                let cross_size = if is_row { self.layout_items[idx].size[1] } else { self.layout_items[idx].size[0] };

                let cross_offset = match fc.align {
                    1 => 0.0,
                    2 => fc.container_cross - cross_size,
                    3 => (fc.container_cross - cross_size) / 2.0,
                    _ => 0.0,
                };

                let final_main_offset = if is_reverse {
                    fc.available_main - main_offset - final_main
                } else {
                    main_offset
                };

                if is_row {
                    self.layout_items[idx].pos[0] = fc.container_pos[0] + fc.container_padding[3] + final_main_offset;
                    self.layout_items[idx].pos[1] = fc.container_pos[1] + fc.container_padding[0] + cross_offset;
                    if fc.align == 0 { self.layout_items[idx].size[1] = fc.container_cross; }
                    self.layout_items[idx].size[0] = final_main;
                } else {
                    self.layout_items[idx].pos[1] = fc.container_pos[1] + fc.container_padding[0] + final_main_offset;
                    self.layout_items[idx].pos[0] = fc.container_pos[0] + fc.container_padding[3] + cross_offset;
                    if fc.align == 0 { self.layout_items[idx].size[0] = fc.container_cross; }
                    self.layout_items[idx].size[1] = final_main;
                }

                main_offset += final_main + fc.gap;
            }

            // justify-content 调整
            if sorted.len() > 1 && remaining > 0.0 {
                let extra_space = remaining;
                let (shift, extra_per_item) = match fc.justify {
                    1 => (extra_space, 0.0),
                    2 => (extra_space / 2.0, 0.0),
                    3 => (0.0, extra_space / (sorted.len() as f32)),
                    4 => (extra_space / (2.0 * sorted.len() as f32), extra_space / (sorted.len() as f32)),
                    5 => (0.0, extra_space / ((sorted.len() + 1) as f32)),
                    _ => (0.0, 0.0),
                };

                let mut cumulative_shift = shift;
                for &idx in &sorted {
                    if is_row {
                        self.layout_items[idx].pos[0] += cumulative_shift;
                    } else {
                        self.layout_items[idx].pos[1] += cumulative_shift;
                    }
                    cumulative_shift += extra_per_item;
                }
            }
        }
    }

    /// Float 浮动布局 — 将 float: left/right 元素定位到容器一侧
    fn compute_float_layout(&mut self) {
        // 建立 dom_id → 索引映射
        let idx_map: std::collections::HashMap<usize, usize> = self.layout_items.iter()
            .enumerate()
            .map(|(i, it)| (it.dom_id as usize, i))
            .collect();

        // 每个容器的浮动跟踪
        struct FloatTrack {
            left_max_x: f32,   // 当前左侧浮动最高 x 位置
            right_min_x: f32,  // 当前右侧浮动最低 x 位置
            left_max_y: f32,   // 左侧浮动最底部 y
            right_max_y: f32,  // 右侧浮动最底部 y
        }

        let mut float_tracks: std::collections::HashMap<usize, FloatTrack> = std::collections::HashMap::new();

        // 收集所有 float 元素
        for i in 0..self.layout_items.len() {
            let float_type = self.layout_items[i].float;
            if float_type == 0 { continue; } // none

            let dom_id = self.layout_items[i].dom_id as usize;
            let parent_id = match self.dom_tree.get_node(dom_id) {
                Some(n) => n.parent,
                None => continue,
            };
            let pid = match parent_id { Some(p) => p, None => continue };
            let container_idx = match idx_map.get(&pid) { Some(&i) => i, None => continue };

            let container = &self.layout_items[container_idx];
            let container_width = container.size[0];
            let container_left = container.pos[0] + container.padding[3];
            let container_top = container.pos[1] + container.padding[0];

            let track = float_tracks.entry(pid).or_insert(FloatTrack {
                left_max_x: container_left,
                right_min_x: container_left + container_width,
                left_max_y: container_top,
                right_max_y: container_top,
            });

            let item = &mut self.layout_items[i];
            let item_w = item.size[0];

            if float_type == 1 { // float: left
                item.pos[0] = track.left_max_x;
                item.pos[1] = track.left_max_y;
                track.left_max_x += item_w + item.margin[1] + item.margin[3]; // 向右推进
                track.left_max_y = track.left_max_y.max(item.pos[1] + item.size[1]);
            } else { // float: right
                item.pos[0] = track.right_min_x - item_w;
                item.pos[1] = track.right_max_y;
                track.right_min_x -= item_w + item.margin[1] + item.margin[3]; // 向左推进
                track.right_max_y = track.right_max_y.max(item.pos[1] + item.size[1]);
            }
        }

        // 处理 clear 属性 — 将 clear 元素推到浮动区域下方
        for i in 0..self.layout_items.len() {
            let clear_val = self.layout_items[i].clear;
            if clear_val == 0 { continue; }

            let dom_id = self.layout_items[i].dom_id as usize;
            let parent_id = match self.dom_tree.get_node(dom_id) {
                Some(n) => n.parent,
                None => continue,
            };
            let pid = match parent_id { Some(p) => p, None => continue };

            if let Some(track) = float_tracks.get(&pid) {
                let min_clear_y = match clear_val {
                    1 => track.left_max_y,  // clear: left
                    2 => track.right_max_y, // clear: right
                    _ => track.left_max_y.max(track.right_max_y), // clear: both
                };
                if self.layout_items[i].pos[1] < min_clear_y {
                    self.layout_items[i].pos[1] = min_clear_y;
                }
            }
        }
    }

    /// Inline 格式化 — 将 inline 元素水平排列在容器中，超出换行
    fn compute_inline_layout(&mut self) {
        // 建立 dom_id → 索引映射
        let idx_map: std::collections::HashMap<usize, usize> = self.layout_items.iter()
            .enumerate()
            .map(|(i, it)| (it.dom_id as usize, i))
            .collect();

        // 按容器分组处理 inline 子元素
        let container_ids: Vec<usize> = self.layout_items.iter()
            .map(|it| it.dom_id as usize)
            .collect();

        for container_dom_id in &container_ids {
            // 找出此容器的 inline 子元素
            let mut inline_children: Vec<usize> = Vec::new();
            for i in 0..self.layout_items.len() {
                let item = &self.layout_items[i];
                if item.is_inline == 0 || item.is_valid == 0 || item.is_hide == 1 {
                    continue;
                }
                let is_child = self.dom_tree.get_node(item.dom_id as usize)
                    .and_then(|n| n.parent)
                    .map_or(false, |p| p == *container_dom_id);
                if is_child {
                    inline_children.push(i);
                }
            }

            if inline_children.is_empty() {
                continue;
            }

            // 找容器的 LayoutItem
            let container_idx = match idx_map.get(container_dom_id) { Some(&i) => i, None => continue };
            let container = &self.layout_items[container_idx];
            let container_left = container.pos[0] + container.padding[3];
            let container_top = container.pos[1] + container.padding[0];
            let container_width = container.size[0];

            // 水平排列 inline 元素
            let mut cursor_x = container_left;
            let mut cursor_y = container_top;
            let mut line_max_height = 0.0f32;

            for &idx in &inline_children {
                let item_w = self.layout_items[idx].size[0];
                let item_h = self.layout_items[idx].size[1];

                // 如果放不下，换行
                if cursor_x + item_w > container_left + container_width && cursor_x > container_left {
                    cursor_x = container_left;
                    cursor_y += line_max_height;
                    line_max_height = 0.0;
                }

                self.layout_items[idx].pos[0] = cursor_x;
                self.layout_items[idx].pos[1] = cursor_y;
                cursor_x += item_w;
                line_max_height = line_max_height.max(item_h);
            }
        }
    }

    /// 基于 DOM 祖先的 overflow 设置计算裁剪矩形
    fn compute_clip_rects(&mut self) {
        if self.layout_items.is_empty() {
            return;
        }

        // 收集每个 DOM 节点的 LayoutItem 索引
        let mut dom_to_item: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
        for (idx, item) in self.layout_items.iter().enumerate() {
            dom_to_item.insert(item.dom_id as usize, idx);
        }

        // 为每个 LayoutItem 计算裁剪矩形
        for i in 0..self.layout_items.len() {
            let mut clip: Option<[f32; 4]> = None;
            let mut current_id = self.layout_items[i].dom_id as usize;

            // 向上遍历 DOM 祖先链
            loop {
                let node = match self.dom_tree.get_node(current_id) {
                    Some(n) => n,
                    None => break,
                };

                let parent_id = match node.parent {
                    Some(id) => id,
                    None => break,
                };

                // 查找父节点的 LayoutItem
                if let Some(&parent_idx) = dom_to_item.get(&parent_id) {
                    let parent = &self.layout_items[parent_idx];
                    let parent_overflow_x = parent.overflow_x.max(parent.overflow);
                    let parent_overflow_y = parent.overflow_y.max(parent.overflow);

                    // 只有非 visible (非 0) 才需要裁剪
                    if parent_overflow_x != 0 || parent_overflow_y != 0 {
                        let parent_x = parent.pos[0];
                        let parent_y = parent.pos[1];
                        let parent_w = parent.size[0];
                        let parent_h = parent.size[1];

                        let new_clip = [parent_x, parent_y, parent_w, parent_h];
                        clip = Some(match clip {
                            Some(existing) => Self::intersect_rects(existing, new_clip),
                            None => new_clip,
                        });
                    }
                }

                current_id = parent_id;
            }

            // 应用裁剪矩形
            if let Some(rect) = clip {
                self.layout_items[i].clip_rect = rect;
            }
        }
    }

    /// 求两个矩形的交集
    fn intersect_rects(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
        let ax = a[0]; let ay = a[1]; let aw = a[2]; let ah = a[3];
        let bx = b[0]; let by = b[1]; let bw = b[2]; let bh = b[3];

        let x = ax.max(bx);
        let y = ay.max(by);
        let w = (ax + aw).min(bx + bw) - x;
        let h = (ay + ah).min(by + bh) - y;

        if w > 0.0 && h > 0.0 {
            [x, y, w, h]
        } else {
            [0.0, 0.0, 0.0, 0.0] // 完全被裁剪掉
        }
    }

    // ── 动画 ──

    /// 推进动画时间，更新动画状态
    pub fn advance_animation(&mut self, dt: f32) {
        self.animation_time += dt;

        // 初始化动画状态（基于 CSS animation 属性）
        if self.animation_states.is_empty() && !self.stylesheet.keyframes.is_empty() {
            for item in &self.layout_items {
                let _dom_id = item.dom_id as usize;
                for (name, _) in &self.stylesheet.keyframes {
                    self.animation_states.push(AnimState::new(name));
                    break; // 简化：每个元素只绑定第一个动画
                }
            }
        }

        // 推进所有动画
        for state in &mut self.animation_states {
            state.advance(dt);
        }

        // 应用动画值到布局项
        if !self.animation_states.is_empty() && !self.layout_items.is_empty() {
            for (i, state) in self.animation_states.iter().enumerate() {
                if !state.running && state.progress() == 0.0 {
                    continue;
                }
                if i >= self.layout_items.len() {
                    break;
                }
                let p = state.progress();
                if let Some(keyframes) = self.stylesheet.keyframes.get(&state.name) {
                    // 尝试插值可动画属性
                    if let Some(val) = interpolate_keyframes(keyframes, "opacity", p) {
                        if let Ok(v) = val.trim_end_matches("px").parse::<f32>() {
                            self.layout_items[i].opacity = v.clamp(0.0, 1.0);
                        }
                    }
                    if let Some(val) = interpolate_keyframes(keyframes, "width", p) {
                        self.layout_items[i].size[0] = val.trim_end_matches("px").parse::<f32>().unwrap_or(self.layout_items[i].size[0]);
                    }
                    if let Some(val) = interpolate_keyframes(keyframes, "height", p) {
                        self.layout_items[i].size[1] = val.trim_end_matches("px").parse::<f32>().unwrap_or(self.layout_items[i].size[1]);
                    }
                }
            }
        }
    }

    /// 将 RGBA 数据转换为 PNG 字节
    fn rgba_to_png(&self, rgba_data: Vec<u8>, width: u32, height: u32) -> Vec<u8> {
        let image = RgbaImage::from_raw(width, height, rgba_data)
            .expect("Invalid image data");
        
        let mut png_bytes = Vec::new();
        image.write_to(&mut std::io::Cursor::new(&mut png_bytes), ImageFormat::Png)
            .expect("Failed to write PNG");
        
        png_bytes
    }

    // ── 增量更新 ──

    /// 标记脏位（防抖合并：按位或合并多次标记）
    fn mark_dirty(&mut self, bits: u8) {
        self.dirty_flags |= bits;
    }

    /// 提交并应用所有待处理的变更，做一次统一布局更新
    fn commit(&mut self) {
        let flags = self.dirty_flags;
        if flags == DIRTY_NONE {
            return;
        }
        self.dirty_flags = DIRTY_NONE;

        log::info!(
            "Commit: flags={:04b} (html={} css={} style={} viewport={})",
            flags,
            (flags & DIRTY_HTML) != 0,
            (flags & DIRTY_CSS) != 0,
            (flags & DIRTY_INLINE_STYLE) != 0,
            (flags & DIRTY_VIEWPORT) != 0,
        );

        // 1. 处理 HTML 变更
        if (flags & DIRTY_HTML) != 0 {
            if let Some(ref html) = self.pending_html {
                let mut parser = HtmlParser::new(html);
                match parser.parse() {
                    Ok(tree) => {
                        self.dom_tree = tree;
                        log::info!("HTML committed: {} nodes", self.dom_tree.len());
                    }
                    Err(e) => {
                        log::error!("HTML commit failed: {}", e);
                    }
                }
                self.pending_html = None;
            }
        }

        // 2. 处理视口变更
        if (flags & DIRTY_VIEWPORT) != 0 {
            if let Some((w, h)) = self.pending_viewport {
                self.width = w;
                self.height = h;
                self.pending_viewport = None;
                log::info!("Viewport committed: {}x{}", w, h);
            }
        }

        // 3. 只在需要时重新解析样式表
        if (flags & DIRTY_CSS) != 0 {
            if let Some(ref css) = self.pending_css {
                let mut combined = StyleSheet::parse(UA_STYLESHEET);
                let user_rules = StyleSheet::parse(css);
                combined.rules.extend(user_rules.rules);
                self.stylesheet = combined;
                self.style_matcher = Some(StyleMatcher::new(self.stylesheet.clone()));
                self.pending_css = None;
                log::info!("CSS committed");
            } else {
                // clear_css 路径：重置到 UA-only
                self.stylesheet = StyleSheet::parse(UA_STYLESHEET);
                self.style_matcher = Some(StyleMatcher::new(self.stylesheet.clone()));
                log::info!("CSS reset to UA defaults");
            }
        }

        // 4. 做一次统一的布局更新（如果 layout 相关的内容变了）
        if (flags & (DIRTY_HTML | DIRTY_CSS | DIRTY_INLINE_STYLE | DIRTY_VIEWPORT)) != 0 {
            self.update_layout();
        }
    }
}

impl WebNativeBridge for Engine {
    fn new(width: u32, height: u32) -> Self
    where
        Self: Sized,
    {
        // 解析 UA 默认样式表
        let ua_stylesheet = StyleSheet::parse(UA_STYLESHEET);
        let style_matcher = Some(StyleMatcher::new(ua_stylesheet.clone()));

        Engine {
            width,
            height,
            device: None,
            queue: None,
            dom_tree: DomTree::new(),
            stylesheet: ua_stylesheet,
            style_matcher,
            layout_items: Vec::new(),
            render_pipeline: None,
            layout_compute: None,
            click_handlers: HashMap::new(),
            form_handlers: HashMap::new(),
            window_open_handlers: Vec::new(),
            current_url: String::new(),
            animation_time: 0.0,
            animation_states: Vec::new(),
            dirty_flags: DIRTY_NONE,
            pending_html: None,
            pending_css: None,
            pending_viewport: None,
        }
    }

    /// 设置页面 HTML
    fn set_html(&mut self, html: &str) {
        self.pending_html = Some(html.to_string());
        self.mark_dirty(DIRTY_HTML | DIRTY_LAYOUT);
        log::info!("HTML queued for update");
    }

    /// 按 CSS 选择器查找第一个元素，返回 DOM 节点 ID
    #[inline]
    fn query(&self, selector: &str) -> Option<usize> {
        let selector = selector.trim();

        if selector.starts_with('#') {
            self.dom_tree.query_by_id(selector)
        } else if selector.starts_with('.') {
            let results = self.dom_tree.query_by_class(selector);
            results.first().copied()
        } else {
            let results = self.dom_tree.query_by_tag(selector);
            results.first().copied()
        }
    }

    /// 按 CSS 选择器查找所有匹配元素
    #[inline]
    fn query_all(&self, selector: &str) -> Vec<usize> {
        let selector = selector.trim();

        if selector == "*" {
            // 通配符选择器：返回所有节点
            self.dom_tree.get_all_nodes()
        } else if selector.starts_with('#') {
            if let Some(id) = self.dom_tree.query_by_id(selector) {
                vec![id]
            } else {
                Vec::new()
            }
        } else if selector.starts_with('.') {
            self.dom_tree.query_by_class(selector)
        } else {
            self.dom_tree.query_by_tag(selector)
        }
    }

    /// 获取元素标签名
    #[inline]
    fn tag_name(&self, node_id: usize) -> Option<String> {
        self.dom_tree.get_node(node_id).map(|n| n.tag_name.clone())
    }

    /// 获取元素属性
    #[inline]
    fn get_attr(&self, node_id: usize, name: &str) -> Option<String> {
        self.dom_tree.get_node(node_id).and_then(|n| n.get_attr(name).cloned())
    }

    /// 设置元素属性
    #[inline]
    fn set_attr(&mut self, node_id: usize, name: &str, value: &str) {
        if let Some(node) = self.dom_tree.get_node_mut(node_id) {
            let old_value = node.get_attr(name).cloned();
            node.set_attr(name.to_string(), value.to_string());

            // class/id 属性变更会影响 CSS 选择器匹配，标记需要重新布局
            if name == "class" || name == "id" || name == "style" {
                let changed = match (&old_value, value) {
                    (Some(old), new) => old != new,
                    (None, "") => false,
                    (None, _) => true,
                };
                if changed {
                    self.mark_dirty(DIRTY_INLINE_STYLE | DIRTY_LAYOUT);
                    log::info!("Attribute '{}' changed on node {}, layout queued", name, node_id);
                }
            }
        }
    }

    /// 获取元素文本内容
    #[inline]
    fn text(&self, node_id: usize) -> Option<String> {
        self.dom_tree.get_node(node_id).and_then(|n| n.text_content.clone())
    }

    /// 获取父节点 ID
    #[inline]
    fn parent_node(&self, node_id: usize) -> Option<usize> {
        self.dom_tree.get_node(node_id).and_then(|n| n.parent)
    }

    fn get_rect(&self, selector: &str) -> Option<LayoutRect> {
        // 获取前确保布局是最新的（查询不走 &mut，假设调用前 render/commit）
        // get_rect 是 &self，不用 commit
        let node_id = self.query(selector)?;

        // 查找对应的布局项
        for item in &self.layout_items {
            if item.dom_id == node_id as u32 {
                return Some(LayoutRect {
                    x: item.pos[0],
                    y: item.pos[1],
                    width: item.size[0],
                    height: item.size[1],
                });
            }
        }

        None
    }

    fn all_rects(&self) -> Vec<LayoutNode> {
        let mut nodes = Vec::new();

        for item in &self.layout_items {
            if item.is_valid == 0 || item.is_hide == 1 {
                continue;
            }

            let tag_name = self.tag_name(item.dom_id as usize).unwrap_or_else(|| "div".to_string());

            let bg_color = item.bg_color;
            let background = Some(Color {
                r: (bg_color[0] * 255.0) as u8,
                g: (bg_color[1] * 255.0) as u8,
                b: (bg_color[2] * 255.0) as u8,
                a: (bg_color[3] * 255.0) as u8,
            });

            nodes.push(LayoutNode {
                dom_node: item.dom_id as usize,
                tag_name,
                x: item.pos[0],
                y: item.pos[1],
                width: item.size[0],
                height: item.size[1],
                background,
            });
        }

        nodes
    }

    fn hit_test(&self, x: f32, y: f32) -> Option<LayoutNode> {
        // 按 z-index 倒序遍历（后面的元素在上层）
        let mut sorted_items: Vec<&LayoutItem> = self.layout_items.iter().collect();
        sorted_items.sort_by(|a, b| {
            b.z_index.partial_cmp(&a.z_index).unwrap_or(std::cmp::Ordering::Equal)
        });

        for item in sorted_items {
            if item.hit_test(x, y) {
                let tag_name = self.tag_name(item.dom_id as usize).unwrap_or_else(|| "div".to_string());

                let bg_color = item.bg_color;
                let background = Some(Color {
                    r: (bg_color[0] * 255.0) as u8,
                    g: (bg_color[1] * 255.0) as u8,
                    b: (bg_color[2] * 255.0) as u8,
                    a: (bg_color[3] * 255.0) as u8,
                });

                return Some(LayoutNode {
                    dom_node: item.dom_id as usize,
                    tag_name,
                    x: item.pos[0],
                    y: item.pos[1],
                    width: item.size[0],
                    height: item.size[1],
                    background,
                });
            }
        }

        None
    }

    fn set_css(&mut self, css_text: &str) {
        self.pending_css = Some(css_text.to_string());
        self.mark_dirty(DIRTY_CSS | DIRTY_LAYOUT);
        log::info!("CSS queued for update");
    }

    fn set_style(&mut self, selector: &str, property: &str, value: &str) {
        if let Some(node_id) = self.query(selector) {
            if let Some(node) = self.dom_tree.get_node_mut(node_id) {
                let current_style = node.get_attr("style").unwrap_or(&String::new()).clone();
                let new_style = if current_style.is_empty() {
                    format!("{}: {}", property, value)
                } else {
                    format!("{}; {}: {}", current_style, property, value)
                };
                node.set_attr("style".to_string(), new_style);
                self.mark_dirty(DIRTY_INLINE_STYLE | DIRTY_LAYOUT);
                log::info!("Style queued for {}: {} = {}", selector, property, value);
            }
        }
    }

    fn clear_css(&mut self) {
        // 清除 pending_css (设为 None 表示 commit 时重置到 UA-only)
        self.pending_css = None;
        self.mark_dirty(DIRTY_CSS | DIRTY_LAYOUT);
        log::info!("CSS clear queued");
    }

    fn flush(&mut self) {
        self.commit();
        log::info!("Flush committed");
    }

    fn render(&mut self) -> Vec<u8> {
        // 提交待处理的变更（防抖合并：多次变更只做一次布局）
        self.commit();

        log::info!("Starting render process");

        // 确保 WebGPU 已初始化
        if self.device.is_none() || self.queue.is_none() {
            if let Err(e) = self.init_webgpu_sync() {
                log::error!("WebGPU initialization failed: {}", e);
                return Vec::new();
            }
        }

        let device = self.device.as_ref().unwrap();
        let queue = self.queue.as_ref().unwrap();

        // 初始化渲染管线（首次调用时）
        if self.render_pipeline.is_none() {
            let mut pipeline = RenderPipelineWrapper::new(device.clone(), queue.clone());
            if let Err(e) = pipeline.initialize(self.width, self.height) {
                log::error!("Pipeline initialization failed: {}", e);
                return Vec::new();
            }
            self.render_pipeline = Some(pipeline);
        }

        // 初始化 GPU 布局计算器（首次调用时）
        if self.layout_compute.is_none() {
            match LayoutCompute::new(device.clone(), queue.clone()) {
                Ok(lc) => {
                    self.layout_compute = Some(lc);
                    log::info!("GPU layout compute initialized");
                }
                Err(e) => {
                    log::warn!("GPU layout compute init failed (CPU fallback): {}", e);
                }
            }
        }

        let pipeline = self.render_pipeline.as_mut().unwrap();

        // 创建输出纹理
        let texture_extent = wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        };

        let output_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Output Texture"),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let texture_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 更新 uniform 数据
        pipeline.update_uniforms(self.width, self.height, self.animation_time);

        // 使用渲染管线绘制布局项
        if let Err(e) = pipeline.render(&texture_view, &self.layout_items, self.width, self.height) {
            log::error!("Pipeline render failed: {}", e);
            return Vec::new();
        }

        // 创建输出缓冲区
        let bytes_per_row = (self.width * 4) as u32;
        let buffer_size = bytes_per_row * self.height;

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: buffer_size as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 创建命令编码器用于复制
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Copy Encoder"),
        });

        // 复制纹理到缓冲区
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(self.height),
                },
            },
            texture_extent,
        );

        let command_buffer = encoder.finish();
        queue.submit(Some(command_buffer));

        // 读取缓冲区内容
        let buffer_slice = output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        device.poll(wgpu::Maintain::Wait);

        if let Ok(result) = rx.recv() {
            if let Ok(()) = result {
                let rgba_data = buffer_slice.get_mapped_range().to_vec();
                log::info!("Render completed: {} bytes", rgba_data.len());
                // 转换为 PNG
                return self.rgba_to_png(rgba_data, self.width, self.height);
            }
        }

        log::error!("Failed to read render output");
        Vec::new()
    }

    fn on_click(&mut self, selector: &str, handler: crate::bridge::EventHandler) {
        if let Some(node_id) = self.query(selector) {
            // 将 Box<dyn FnMut> 包装为 Rc<RefCell<dyn FnMut>>
            let wrapped: Rc<RefCell<dyn FnMut(f32, f32)>> = Rc::new(RefCell::new(handler));
            self.click_handlers
                .entry(node_id)
                .or_insert_with(Vec::new)
                .push(wrapped);
            log::info!("Click handler registered for {}", selector);
        }
    }

    fn on_form_submit(&mut self, selector: &str, handler: crate::bridge::FormHandler) {
        let wrapped: Rc<RefCell<dyn FnMut(HashMap<String, String>)>> = Rc::new(RefCell::new(handler));
        self.form_handlers.insert(selector.to_string(), wrapped);
        log::info!("Form submit handler registered for {}", selector);
    }

    fn on_window_open(&mut self, handler: crate::bridge::WindowOpenHandler) {
        let wrapped: Rc<RefCell<dyn FnMut(&str) -> bool>> = Rc::new(RefCell::new(handler));
        self.window_open_handlers.push(wrapped);
        log::info!("Window open handler registered");
    }

    fn handle_click(&mut self, x: f32, y: f32) -> bool {
        // 执行 hit_test 找到被点击的元素
        if let Some(node) = self.hit_test(x, y) {
            log::info!("Click at ({}, {}) hit node {}", x, y, node.tag_name);

            // 触发该元素的点击处理器
            if let Some(handlers) = self.click_handlers.get(&node.dom_node) {
                for handler in handlers {
                    handler.borrow_mut()(x, y);
                }
                return true;
            }

            // 事件冒泡：检查父元素
            let mut current_node = node.dom_node;
            while let Some(parent_id) = self.parent_node(current_node) {
                if let Some(handlers) = self.click_handlers.get(&parent_id) {
                    for handler in handlers {
                        handler.borrow_mut()(x, y);
                    }
                    return true;
                }
                current_node = parent_id;
            }
        }
        false
    }

    fn handle_form_submit(&mut self, form_selector: &str) {
        if let Some(handler) = self.form_handlers.get(form_selector) {
            // 收集表单数据
            let mut form_data = HashMap::new();

            // 查找表单内的输入元素
            if let Some(_form_id) = self.query(form_selector) {
                // 查找所有输入元素 (简化实现)
                for input_id in self.query_all(&format!("{} input, {} select, {} textarea", form_selector, form_selector, form_selector)) {
                    if let Some(name) = self.get_attr(input_id, "name") {
                        let value = self.get_attr(input_id, "value").unwrap_or_default();
                        form_data.insert(name, value);
                    }
                }
            }

            handler.borrow_mut()(form_data);
        }
    }

    fn handle_window_open(&mut self, url: &str) -> bool {
        for handler in &self.window_open_handlers {
            if handler.borrow_mut()(url) {
                return true;
            }
        }
        false
    }

    fn set_viewport(&mut self, width: u32, height: u32) {
        self.pending_viewport = Some((width, height));
        self.mark_dirty(DIRTY_VIEWPORT | DIRTY_LAYOUT);
        log::info!("Viewport change queued: {}x{}", width, height);
    }

    fn viewport(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn navigate(&mut self, url: &str) -> Result<(), String> {
        // 更新当前 URL
        self.current_url = url.to_string();
        log::info!("Navigated to: {}", url);
        // TODO: 实际获取 URL 内容并解析 HTML
        Ok(())
    }

    fn current_url(&self) -> String {
        self.current_url.clone()
    }

    fn http_get(&mut self, url: &str) -> Result<HttpResponse, String> {
        log::info!("HTTP GET: {}", url);
        let response = ureq::get(url)
            .call()
            .map_err(|e| format!("HTTP GET failed: {}", e))?;

        let status_code = response.status();
        let mut resp = HttpResponse::new(status_code);

        // 复制响应头
        for header_name in response.headers_names() {
            if let Some(value) = response.header(&header_name) {
                resp.set_header(&header_name, value);
            }
        }

        // 读取响应体
        let mut body: Vec<u8> = Vec::new();
        response.into_reader().read_to_end(&mut body)
            .map_err(|e| format!("Failed to read response body: {}", e))?;
        resp.set_body(body);

        log::info!("HTTP GET completed: status={}", status_code);
        Ok(resp)
    }

    fn http_post(
        &mut self,
        url: &str,
        body: &[u8],
        content_type: &str,
    ) -> Result<HttpResponse, String> {
        log::info!("HTTP POST: {} ({} bytes)", url, body.len());
        let response = ureq::post(url)
            .set("Content-Type", content_type)
            .send_bytes(body)
            .map_err(|e| format!("HTTP POST failed: {}", e))?;

        let status_code = response.status();
        let mut resp = HttpResponse::new(status_code);

        for header_name in response.headers_names() {
            if let Some(value) = response.header(&header_name) {
                resp.set_header(&header_name, value);
            }
        }

        let mut response_body: Vec<u8> = Vec::new();
        response.into_reader().read_to_end(&mut response_body)
            .map_err(|e| format!("Failed to read response body: {}", e))?;
        resp.set_body(response_body);

        log::info!("HTTP POST completed: status={}", status_code);
        Ok(resp)
    }

    fn download_file(&mut self, url: &str, path: &str) -> Result<u64, String> {
        log::info!("Downloading: {} -> {}", url, path);
        let response = self.http_get(url)?;
        let data = response.body;
        std::fs::write(path, &data)
            .map_err(|e| format!("Failed to write file '{}': {}", path, e))?;
        let size = data.len() as u64;
        log::info!("Download completed: {} bytes -> {}", size, path);
        Ok(size)
    }

    fn write_file(&mut self, path: &str, data: &[u8]) -> Result<(), String> {
        log::info!("Writing file: {} ({} bytes)", path, data.len());
        std::fs::write(path, data)
            .map_err(|e| format!("Failed to write file '{}': {}", path, e))?;
        log::info!("File written: {}", path);
        Ok(())
    }

    fn read_file(&mut self, path: &str) -> Result<Vec<u8>, String> {
        log::info!("Reading file: {}", path);
        let data = std::fs::read(path)
            .map_err(|e| format!("Failed to read file '{}': {}", path, e))?;
        log::info!("File read: {} ({} bytes)", path, data.len());
        Ok(data)
    }
}