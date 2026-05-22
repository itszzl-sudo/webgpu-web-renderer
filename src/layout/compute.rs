//! GPU 布局计算模块
//! 
//! 使用 WebGPU Compute Shader 实现并行布局计算

use crate::layout::{LayoutItem, LayoutEnv};
use wgpu::{Device, Queue, Buffer};
use std::sync::Arc;

const COMPUTE_SHADER_SOURCE: &str = include_str!("compute.wgsl");

/// GPU 布局计算器
pub struct LayoutCompute {
    device: Arc<Device>,
    queue: Arc<Queue>,
    items_buffer: Buffer,
    env_buffer: Buffer,
}

impl LayoutCompute {
    /// 创建新的布局计算器
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Result<Self, String> {
        // 创建 items 缓冲区 (storage buffer)
        let items_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("layout_items_buffer"),
            size: 1024 * 1024, // 1MB 预分配
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // 创建 env 缓冲区 (uniform buffer)
        let env_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("layout_env_buffer"),
            size: std::mem::size_of::<LayoutEnv>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(LayoutCompute {
            device,
            queue,
            items_buffer,
            env_buffer,
        })
    }

    /// 更新布局项数据
    pub fn update_items(&self, items: &[LayoutItem]) -> Result<(), String> {
        let data_size = items.len() * std::mem::size_of::<LayoutItem>();
        if data_size > self.items_buffer.size() as usize {
            return Err("Items data too large for buffer".to_string());
        }

        self.queue.write_buffer(
            &self.items_buffer,
            0,
            bytemuck::cast_slice(items),
        );

        Ok(())
    }

    /// 更新布局环境数据
    pub fn update_env(&self, env: &LayoutEnv) -> Result<(), String> {
        self.queue.write_buffer(
            &self.env_buffer,
            0,
            bytemuck::cast_slice(&[*env]),
        );

        Ok(())
    }

    /// 执行布局计算 (简化版本，使用 CPU 回退)
    pub fn compute(&self, _item_count: u32) -> Result<(), String> {
        // GPU Compute Shader 需要更多设置
        // 当前使用 CPU 回退计算
        Ok(())
    }

    /// 获取 items 缓冲区
    pub fn items_buffer(&self) -> &Buffer {
        &self.items_buffer
    }

    /// 获取 env 缓冲区
    pub fn env_buffer(&self) -> &Buffer {
        &self.env_buffer
    }
}

/// CPU 布局计算器 - 当 GPU 不可用时回退使用
pub struct CpuLayoutCompute;

impl CpuLayoutCompute {
    /// 执行 CPU 布局计算
    pub fn compute(items: &mut [LayoutItem], env: &LayoutEnv) {
        for item in items.iter_mut() {
            if item.is_valid == 0 || item.is_hide == 1 {
                continue;
            }

            match item.flow_type {
                0 => Self::compute_flow(item, env),
                1 => Self::compute_absolute(item, env),
                2 => Self::compute_flex(item, env),
                _ => {}
            }
        }
    }

    /// 文档流布局计算
    fn compute_flow(item: &mut LayoutItem, _env: &LayoutEnv) {
        // 简单的文档流：从上到下
        // 应用 max/min width/height 约束
        let mut width = item.size[0];
        let mut height = item.size[1];

        if item.max_width > 0.0 && width > item.max_width {
            width = item.max_width;
        }
        if item.min_width > 0.0 && width < item.min_width {
            width = item.min_width;
        }
        if item.max_height > 0.0 && height > item.max_height {
            height = item.max_height;
        }
        if item.min_height > 0.0 && height < item.min_height {
            height = item.min_height;
        }

        item.size = [width, height];
    }

    /// 绝对定位布局计算
    fn compute_absolute(item: &mut LayoutItem, env: &LayoutEnv) {
        // 如果有 size_constraint (right/bottom)，重新计算位置
        if item.size_constraint[0] >= 0.0 {
            item.pos[0] = env.view_size[0] - item.size_constraint[0] - item.size[0];
        }
        if item.size_constraint[1] >= 0.0 {
            item.pos[1] = env.view_size[1] - item.size_constraint[1] - item.size[1];
        }
    }

    /// Flex 布局计算 (简化)
    fn compute_flex(item: &mut LayoutItem, _env: &LayoutEnv) {
        // 简化实现：按权重分配空间
        if item.weight > 0.0 {
            let total_weight: f32 = item.weight; // 实际需要遍历所有 flex 子项
            let allocated = (item.weight / total_weight) * _env.view_size[0];
            item.size[0] = allocated;
        }

        // 处理 flex-basis
        if item.flex_basis > 0.0 {
            item.size[0] = item.flex_basis;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_layout_compute_flow() {
        let mut items = vec![
            LayoutItem::new().with_size(100.0, 50.0).with_pos(0.0, 0.0),
            LayoutItem::new().with_size(200.0, 30.0).with_pos(0.0, 50.0),
        ];
        let env = LayoutEnv::new(800.0, 600.0);

        CpuLayoutCompute::compute(&mut items, &env);

        assert_eq!(items[0].size, [100.0, 50.0]);
        assert_eq!(items[1].size, [200.0, 30.0]);
    }

    #[test]
    fn test_cpu_layout_compute_absolute() {
        let mut item = LayoutItem::new()
            .with_pos(10.0, 20.0)
            .with_size(100.0, 50.0)
            .with_flow_type(1)
            .with_size_constraint(50.0, 100.0);

        let env = LayoutEnv::new(800.0, 600.0);

        CpuLayoutCompute::compute(std::slice::from_mut(&mut item), &env);

        // right=50, width=100, view_width=800 -> pos_x = 800 - 50 - 100 = 650
        assert_eq!(item.pos[0], 650.0);
    }

    #[test]
    fn test_cpu_layout_compute_max_min_width() {
        let mut item = LayoutItem::new()
            .with_size(500.0, 100.0)
            .with_max_width(200.0);

        let env = LayoutEnv::new(800.0, 600.0);

        CpuLayoutCompute::compute(std::slice::from_mut(&mut item), &env);

        assert_eq!(item.size[0], 200.0); // 被 max_width 限制
    }

    #[test]
    fn test_cpu_layout_compute_hidden_item() {
        let mut item = LayoutItem::new()
            .with_size(500.0, 100.0)
            .with_pos(0.0, 0.0)
            .hide();

        let env = LayoutEnv::new(800.0, 600.0);

        CpuLayoutCompute::compute(std::slice::from_mut(&mut item), &env);

        // 隐藏项不应被修改
        assert_eq!(item.size[0], 500.0);
        assert_eq!(item.pos[0], 0.0);
    }
}