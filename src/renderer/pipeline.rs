use crate::layout::LayoutItem;
use wgpu::{Device, RenderPipeline, TextureView, TextureFormat};
use wgpu::util::DeviceExt;
use std::sync::Arc;

/// 顶点数据 - 与 WGSL 着色器对应
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],  // [x, y]
    pub uv: [f32; 2],        // [u, v]
    pub tex_idx: u32,         // 纹理索引 (0=无纹理, >0=有纹理)
    pub color: [f32; 4],      // [r, g, b, a]
}

impl Vertex {
    pub fn new(x: f32, y: f32, u: f32, v: f32, tex_idx: u32, color: [f32; 4]) -> Self {
        Vertex {
            position: [x, y],
            uv: [u, v],
            tex_idx,
            color,
        }
    }

    /// 创建纯色顶点 (无纹理)
    pub fn solid(x: f32, y: f32, u: f32, v: f32, r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::new(x, y, u, v, 0, [r, g, b, a])
    }
}

/// 生成矩形顶点 (2个三角形组成)
pub fn generate_rect_vertices(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    tex_idx: u32,
    color: [f32; 4],
) -> [Vertex; 6] {
    let x2 = x + width;
    let y2 = y + height;

    [
        // 第一个三角形 (左上, 右上, 左下)
        Vertex::new(x, y, 0.0, 0.0, tex_idx, color),
        Vertex::new(x2, y, 1.0, 0.0, tex_idx, color),
        Vertex::new(x, y2, 0.0, 1.0, tex_idx, color),
        // 第二个三角形 (右上, 右下, 左下)
        Vertex::new(x2, y, 1.0, 0.0, tex_idx, color),
        Vertex::new(x2, y2, 1.0, 1.0, tex_idx, color),
        Vertex::new(x, y2, 0.0, 1.0, tex_idx, color),
    ]
}

/// Uniform 数据 - 与 WGSL Uniforms 对应
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    pub viewport_size: [f32; 2],  // 8 bytes, offset 0
    pub time: f32,                 // 4 bytes, offset 8
    _pad1: [f32; 1],               // 4 bytes padding (align vec2 to 8)
    pub _pad: [f32; 2],            // 8 bytes, offset 16
}

impl Default for Uniforms {
    fn default() -> Self {
        Uniforms {
            viewport_size: [800.0, 600.0],
            time: 0.0,
            _pad1: [0.0; 1],
            _pad: [0.0; 2],
        }
    }
}

/// 纹理管理器
pub struct TextureManager {
    device: Arc<Device>,
    textures: Vec<wgpu::Texture>,
    texture_views: Vec<wgpu::TextureView>,
    sampler: wgpu::Sampler,
}

impl TextureManager {
    pub fn new(device: Arc<Device>) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        TextureManager {
            device,
            textures: Vec::new(),
            texture_views: Vec::new(),
            sampler,
        }
    }

    /// 创建纯色纹理
    pub fn create_solid_color(&mut self, queue: &wgpu::Queue, r: f32, g: f32, b: f32, a: f32) -> u32 {
        let rgba = vec![
            (r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, (a * 255.0) as u8,
        ];

        self.create_texture_from_rgba(queue, &rgba, 1, 1)
    }

    /// 从 RGBA 数据创建纹理
    pub fn create_texture_from_rgba(&mut self, queue: &wgpu::Queue, rgba_data: &[u8], width: u32, height: u32) -> u32 {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Created Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let tex_idx = self.textures.len() as u32 + 1; // 纹理索引从 1 开始
        self.textures.push(texture);
        self.texture_views.push(view);

        tex_idx
    }

    /// 获取纹理视图
    pub fn get_view(&self, index: usize) -> Option<&wgpu::TextureView> {
        self.texture_views.get(index)
    }

    /// 获取采样器
    pub fn get_sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    /// 获取纹理数量
    pub fn len(&self) -> usize {
        self.textures.len()
    }

    /// 清空所有纹理
    pub fn clear(&mut self) {
        self.textures.clear();
        self.texture_views.clear();
    }
}

/// 渲染管线
pub struct RenderPipelineWrapper {
    device: Arc<Device>,
    queue: Arc<wgpu::Queue>,
    pipeline: Option<RenderPipeline>,
    bind_group: Option<wgpu::BindGroup>,
    uniform_buffer: Option<wgpu::Buffer>,
    texture_manager: TextureManager,
    vertex_buffer: Option<wgpu::Buffer>,
}

impl RenderPipelineWrapper {
    pub fn new(device: Arc<Device>, queue: Arc<wgpu::Queue>) -> Self {
        let texture_manager = TextureManager::new(device.clone());

        RenderPipelineWrapper {
            device,
            queue,
            pipeline: None,
            bind_group: None,
            uniform_buffer: None,
            texture_manager,
            vertex_buffer: None,
        }
    }

    /// 初始化渲染管线
    pub fn initialize(&mut self, width: u32, height: u32) -> Result<(), String> {
        let bind_group_layout = self.create_bind_group_layout()?;
        self.create_uniform_buffer(width, height)?;
        self.create_render_pipeline(&bind_group_layout)?;
        self.create_bind_group(&bind_group_layout)?;

        Ok(())
    }

    /// 创建绑定组布局
    fn create_bind_group_layout(&self) -> Result<wgpu::BindGroupLayout, String> {
        let layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        Ok(layout)
    }

    /// 创建绑定组
    fn create_bind_group(&mut self, layout: &wgpu::BindGroupLayout) -> Result<(), String> {
        let buffer = self.uniform_buffer.as_ref().ok_or("Uniform buffer not created")?;
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        self.bind_group = Some(bind_group);
        Ok(())
    }

    /// 创建 Uniform 缓冲区
    fn create_uniform_buffer(&mut self, width: u32, height: u32) -> Result<(), String> {
        self.uniform_buffer = Some(
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Uniform Buffer"),
                size: std::mem::size_of::<Uniforms>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );

        // 更新 uniform 数据
        let uniforms = Uniforms {
            viewport_size: [width as f32, height as f32],
            time: 0.0,
            _pad1: [0.0; 1],
            _pad: [0.0, 0.0],
        };

        if let Some(buffer) = self.uniform_buffer.as_ref() {
            self.queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[uniforms]));
        }

        Ok(())
    }

    /// 创建渲染管线
    fn create_render_pipeline(&mut self, bind_group_layout: &wgpu::BindGroupLayout) -> Result<(), String> {
        let shader_source = include_str!("render.wgsl");
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Render Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vertex_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: 8,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: 16,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Uint32,
                        },
                        wgpu::VertexAttribute {
                            offset: 20,
                            shader_location: 3,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fragment_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        self.pipeline = Some(pipeline);
        Ok(())
    }

    /// 获取 Uniform 缓冲区引用
    pub fn uniform_buffer(&self) -> Option<&wgpu::Buffer> {
        self.uniform_buffer.as_ref()
    }

    /// 更新 Uniform 数据
    pub fn update_uniforms(&mut self, width: u32, height: u32) {
        let uniforms = Uniforms {
            viewport_size: [width as f32, height as f32],
            time: 0.0,
            _pad1: [0.0; 1],
            _pad: [0.0, 0.0],
        };
        if let Some(buffer) = self.uniform_buffer.as_ref() {
            self.queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[uniforms]));
        }
    }

    /// 获取纹理管理器
    pub fn texture_manager(&mut self) -> &mut TextureManager {
        &mut self.texture_manager
    }

    /// 从 LayoutItem 生成顶点
    pub fn generate_vertices_from_item(item: &LayoutItem) -> [Vertex; 6] {
        generate_rect_vertices(
            item.pos[0],
            item.pos[1],
            item.size[0],
            item.size[1],
            item.tex_idx,
            item.bg_color,
        )
    }

    /// 预创建顶点缓冲区
    pub fn create_vertex_buffer(&mut self, vertex_count: usize) {
        let buffer_size = vertex_count * std::mem::size_of::<Vertex>();
        self.vertex_buffer = Some(
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Vertex Buffer"),
                size: buffer_size as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );
    }

    /// 渲染场景
    pub fn render(
        &mut self,
        view: &TextureView,
        layout_items: &[LayoutItem],
    ) -> Result<(), String> {
        let pipeline = self.pipeline.as_ref().ok_or("Render pipeline not initialized")?;
        let bind_group = self.bind_group.as_ref().ok_or("Bind group not created")?;

        // 生成顶点数据
        let mut vertices = Vec::new();
        for item in layout_items {
            if item.is_valid == 0 || item.is_hide == 1 {
                continue;
            }
            let rect = Self::generate_vertices_from_item(item);
            vertices.extend_from_slice(&rect);
        }

        if vertices.is_empty() {
            return Ok(());
        }

        // 创建顶点缓冲区
        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..vertices.len() as u32, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_creation() {
        let vertex = Vertex::new(0.0, 0.0, 0.0, 0.0, 1, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(vertex.position, [0.0, 0.0]);
        assert_eq!(vertex.tex_idx, 1);
        assert_eq!(vertex.color, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_generate_rect_vertices() {
        let vertices = generate_rect_vertices(10.0, 20.0, 100.0, 50.0, 0, [0.5, 0.5, 0.5, 1.0]);
        
        assert_eq!(vertices.len(), 6);
        assert_eq!(vertices[0].position, [10.0, 20.0]);
        assert_eq!(vertices[1].position, [110.0, 20.0]);
        assert_eq!(vertices[2].position, [10.0, 70.0]);
    }

    #[test]
    fn test_layout_item_to_vertices() {
        let item = LayoutItem::new()
            .with_pos(50.0, 100.0)
            .with_size(200.0, 150.0)
            .with_bg_color(0.0, 0.5, 1.0, 1.0);

        let vertices = RenderPipelineWrapper::generate_vertices_from_item(&item);

        assert_eq!(vertices[0].position, [50.0, 100.0]);
        assert_eq!(vertices[1].position, [250.0, 100.0]);
        assert_eq!(vertices[0].color, [0.0, 0.5, 1.0, 1.0]);
    }

    #[test]
    fn test_uniforms_default() {
        let uniforms = Uniforms::default();
        assert_eq!(uniforms.viewport_size, [800.0, 600.0]);
        assert_eq!(uniforms.time, 0.0);
    }
}