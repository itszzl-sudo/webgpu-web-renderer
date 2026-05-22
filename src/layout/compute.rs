//! GPU 布局计算模块
//! 
//! 使用 WebGPU Compute Shader 实现并行布局计算。
//! 
//! # 并行策略
//! - **尺寸约束** (max/min width/height): 完全并行, 每线程处理一项
//! - **绝对定位**: 完全并行, 每线程独立计算
//! - **Flex 权重收集**: workgroup 共享内存汇总
//! - **文档流 Y 位置**: 串行依赖, 回退到 CPU 计算

use crate::layout::{LayoutItem, LayoutEnv};
use wgpu::{Device, Queue, Buffer, ComputePipeline, BindGroup};
use std::sync::Arc;
use std::mem;

/// 计算汇总结果 (与 WGSL AtomicResult 对齐)
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct AtomicResult {
    total_count: u32,
    total_weight: f32,
}

impl Default for AtomicResult {
    fn default() -> Self {
        AtomicResult {
            total_count: 0,
            total_weight: 0.0,
        }
    }
}

const COMPUTE_SHADER_SOURCE: &str = include_str!("compute.wgsl");

/// GPU 布局计算器
pub struct LayoutCompute {
    device: Arc<Device>,
    queue: Arc<Queue>,
    items_buffer: Buffer,
    env_buffer: Buffer,
    result_buffer: Buffer,
    compute_pipeline: ComputePipeline,
    bind_group: BindGroup,
    max_items: usize,
}

impl LayoutCompute {
    /// 创建新的布局计算器
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Result<Self, String> {
        let max_items = 4096; // 最大支持项数
        let item_buffer_size = (max_items * mem::size_of::<LayoutItem>()) as u64;

        // 创建 items 缓冲区 (storage buffer, 读写)
        let items_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("layout_items_buffer"),
            size: item_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // 创建 env 缓冲区 (uniform buffer)
        let env_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("layout_env_buffer"),
            size: mem::size_of::<LayoutEnv>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 创建 result 缓冲区 (storage buffer, 读写)
        let result_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("layout_result_buffer"),
            size: mem::size_of::<AtomicResult>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // 创建 shader 模块
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Layout Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(COMPUTE_SHADER_SOURCE.into()),
        });

        // 创建 bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Layout Compute Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // 创建 pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Layout Compute Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // 创建 compute pipeline
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Layout Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "layout_compute",
        });

        // 创建 bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Layout Compute Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: items_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: env_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: result_buffer.as_entire_binding(),
                },
            ],
        });

        Ok(LayoutCompute {
            device,
            queue,
            items_buffer,
            env_buffer,
            result_buffer,
            compute_pipeline,
            bind_group,
            max_items,
        })
    }

    /// 更新布局项数据到 GPU
    pub fn update_items(&self, items: &[LayoutItem]) -> Result<(), String> {
        let data_size = items.len() * mem::size_of::<LayoutItem>();
        if data_size > self.items_buffer.size() as usize {
            return Err(format!(
                "Items data too large for buffer: {} > {}",
                data_size, self.items_buffer.size()
            ));
        }

        self.queue.write_buffer(
            &self.items_buffer,
            0,
            bytemuck::cast_slice(items),
        );

        Ok(())
    }

    /// 更新布局环境数据到 GPU
    pub fn update_env(&self, env: &LayoutEnv) -> Result<(), String> {
        self.queue.write_buffer(
            &self.env_buffer,
            0,
            bytemuck::cast_slice(&[*env]),
        );

        Ok(())
    }

    /// 重置结果缓冲区
    fn reset_result(&self) {
        let zeros = AtomicResult::default();
        self.queue.write_buffer(
            &self.result_buffer,
            0,
            bytemuck::cast_slice(&[zeros]),
        );
    }

    /// 执行 GPU 布局计算
    pub fn compute(&self, item_count: u32) -> Result<(), String> {
        if item_count == 0 {
            return Ok(());
        }

        // 重置结果缓冲区
        self.reset_result();

        // 创建命令编码器
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Layout Compute Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Layout Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);

            // 调度: workgroup_size=64, 每线程处理一个 item
            let workgroup_count = (item_count + 63) / 64;
            compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        log::info!("GPU layout compute dispatched: {} items in {} workgroups", 
            item_count, (item_count + 63) / 64);

        Ok(())
    }

    /// 从 GPU 读取布局计算结果
    pub fn read_results(&self, items: &mut [LayoutItem]) -> Result<(), String> {
        if items.is_empty() {
            return Ok(());
        }

        let read_size = (items.len() * mem::size_of::<LayoutItem>()) as u64;

        // 创建 staging buffer 用于读取
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Layout Readback Buffer"),
            size: read_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 创建命令编码器并复制数据
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Layout Readback Encoder"),
        });

        encoder.copy_buffer_to_buffer(
            &self.items_buffer,
            0,
            &staging_buffer,
            0,
            read_size,
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        // 读取映射结果
        let buffer_slice = staging_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        self.device.poll(wgpu::Maintain::Wait);

        match rx.recv() {
            Ok(Ok(())) => {
                let data = buffer_slice.get_mapped_range();
                // 只复制前 items.len() 个元素
                let item_bytes = items.len() * mem::size_of::<LayoutItem>();
                let src: &[u8] = &data[..item_bytes.min(data.len())];
                let dst: &mut [u8] = bytemuck::cast_slice_mut(items);
                dst.copy_from_slice(src);
                drop(data);
                staging_buffer.destroy();
                Ok(())
            }
            _ => {
                staging_buffer.destroy();
                Err("Failed to read back layout results".to_string())
            }
        }
    }

    /// 获取 items 缓冲区引用
    pub fn items_buffer(&self) -> &Buffer {
        &self.items_buffer
    }

    /// 获取 env 缓冲区引用
    pub fn env_buffer(&self) -> &Buffer {
        &self.env_buffer
    }

    /// 获取最大支持的项数
    pub fn max_items(&self) -> usize {
        self.max_items
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
        if item.size_constraint[0] >= 0.0 {
            item.pos[0] = env.view_size[0] - item.size_constraint[0] - item.size[0];
        }
        if item.size_constraint[1] >= 0.0 {
            item.pos[1] = env.view_size[1] - item.size_constraint[1] - item.size[1];
        }
    }

    /// Flex 布局计算 (简化)
    fn compute_flex(item: &mut LayoutItem, _env: &LayoutEnv) {
        if item.weight > 0.0 {
            let total_weight: f32 = item.weight;
            let allocated = (item.weight / total_weight) * _env.view_size[0];
            item.size[0] = allocated;
        }

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

        assert_eq!(item.pos[0], 650.0);
    }

    #[test]
    fn test_cpu_layout_compute_max_min_width() {
        let mut item = LayoutItem::new()
            .with_size(500.0, 100.0)
            .with_max_width(200.0);

        let env = LayoutEnv::new(800.0, 600.0);

        CpuLayoutCompute::compute(std::slice::from_mut(&mut item), &env);

        assert_eq!(item.size[0], 200.0);
    }

    #[test]
    fn test_cpu_layout_compute_hidden_item() {
        let mut item = LayoutItem::new()
            .with_size(500.0, 100.0)
            .with_pos(0.0, 0.0)
            .hide();

        let env = LayoutEnv::new(800.0, 600.0);

        CpuLayoutCompute::compute(std::slice::from_mut(&mut item), &env);

        assert_eq!(item.size[0], 500.0);
        assert_eq!(item.pos[0], 0.0);
    }

    #[test]
    fn test_atomic_result_memory_layout() {
        assert_eq!(mem::size_of::<AtomicResult>(), 8);
    }
}