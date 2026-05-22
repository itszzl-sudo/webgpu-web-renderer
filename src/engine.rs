use crate::bridge::{WebNativeBridge, LayoutRect, LayoutNode, Color};
use crate::dom::tree::{DomTree, DomNode};
use crate::dom::parser::HtmlParser;
use crate::css::parser::StyleSheet;
use crate::css::matcher::StyleMatcher;
use image::{RgbaImage, ImageFormat};
use std::collections::HashMap;

pub struct Engine {
    width: u32,
    height: u32,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    dom_tree: DomTree,
    stylesheet: StyleSheet,
    style_matcher: Option<StyleMatcher>,
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

        self.device = Some(device);
        self.queue = Some(queue);

        log::info!("WebGPU initialized successfully");
        Ok(())
    }

    /// 解析 CSS 并更新样式表
    fn parse_css(&mut self, css_text: &str) {
        self.stylesheet = StyleSheet::parse(css_text);
        self.style_matcher = Some(StyleMatcher::new(self.stylesheet.clone()));
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
}

impl WebNativeBridge for Engine {
    fn new(width: u32, height: u32) -> Self
    where
        Self: Sized,
    {
        let stylesheet = StyleSheet::new();
        let style_matcher = Some(StyleMatcher::new(stylesheet.clone()));

        Engine {
            width,
            height,
            device: None,
            queue: None,
            dom_tree: DomTree::new(),
            stylesheet,
            style_matcher,
        }
    }

    /// 设置页面 HTML
    fn set_html(&mut self, html: &str) {
        let mut parser = HtmlParser::new(html);
        match parser.parse() {
            Ok(tree) => {
                self.dom_tree = tree;
                log::info!("HTML parsed successfully, {} nodes", self.dom_tree.len());
            }
            Err(e) => {
                log::error!("Failed to parse HTML: {}", e);
            }
        }
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
            node.set_attr(name.to_string(), value.to_string());
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
        // TODO: 实现布局查询
        None
    }

    fn all_rects(&self) -> Vec<LayoutNode> {
        // TODO: 实现所有布局节点查询
        Vec::new()
    }

    fn hit_test(&self, _x: f32, _y: f32) -> Option<LayoutNode> {
        // 简化的点击测试实现
        if let Some(node_id) = self.dom_tree.get_root() {
            if let Some(node) = self.dom_tree.get_node(node_id) {
                let tag_name = node.tag_name.clone();
                return Some(LayoutNode {
                    dom_node: node_id,
                    tag_name,
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 20.0,
                    background: None,
                });
            }
        }
        None
    }

    fn set_css(&mut self, css_text: &str) {
        self.parse_css(css_text);
        log::info!("CSS rules updated");
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
                log::info!("Style set for {}: {} = {}", selector, property, value);
            }
        }
    }

    fn clear_css(&mut self) {
        self.stylesheet = StyleSheet::new();
        self.style_matcher = Some(StyleMatcher::new(self.stylesheet.clone()));
        log::info!("CSS cleared");
    }

    fn render(&mut self) -> Vec<u8> {
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

        // 创建输出缓冲区
        let bytes_per_row = (self.width * 4) as u32;
        let buffer_size = bytes_per_row * self.height;

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: buffer_size as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 创建命令编码器
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // 清除纹理为白色背景
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
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
        }

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
                let data = buffer_slice.get_mapped_range().to_vec();
                log::info!("Render completed: {} bytes", data.len());
                return data;
            }
        }

        log::error!("Failed to read render output");
        Vec::new()
    }

    fn on_click(&mut self, _selector: &str, _handler: crate::bridge::EventHandler) {
        log::info!("Click event registered (TODO)");
    }

    fn on_form_submit(&mut self, _selector: &str, _handler: crate::bridge::FormHandler) {
        log::info!("Form submit event registered (TODO)");
    }

    fn on_window_open(&mut self, _handler: crate::bridge::WindowOpenHandler) {
        log::info!("Window.open event registered (TODO)");
    }

    fn handle_click(&mut self, _x: f32, _y: f32) -> bool {
        log::info!("Click handled (TODO)");
        false
    }

    fn handle_form_submit(&mut self, _form_selector: &str) {
        log::info!("Form submit handled (TODO)");
    }

    fn handle_window_open(&mut self, _url: &str) -> bool {
        log::info!("Window open handled (TODO)");
        false
    }

    fn set_viewport(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        // TODO: 通知 WebGPU 更新视口
    }

    fn viewport(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn navigate(&mut self, _url: &str) -> Result<(), String> {
        // TODO: 实现导航
        Ok(())
    }

    fn current_url(&self) -> String {
        // TODO: 返回当前 URL
        String::new()
    }

    fn http_get(&mut self, _url: &str) -> Result<crate::network::HttpResponse, String> {
        // TODO: 实现 HTTP GET
        Err(String::from("Not implemented"))
    }

    fn http_post(
        &mut self,
        _url: &str,
        _body: &[u8],
        _content_type: &str,
    ) -> Result<crate::network::HttpResponse, String> {
        // TODO: 实现 HTTP POST
        Err(String::from("Not implemented"))
    }

    fn download_file(&mut self, _url: &str, _path: &str) -> Result<u64, String> {
        // TODO: 实现文件下载
        Err(String::from("Not implemented"))
    }

    fn write_file(&mut self, _path: &str, _data: &[u8]) -> Result<(), String> {
        // TODO: 实现文件写入
        Err(String::from("Not implemented"))
    }

    fn read_file(&mut self, _path: &str) -> Result<Vec<u8>, String> {
        // TODO: 实现文件读取
        Err(String::from("Not implemented"))
    }
}