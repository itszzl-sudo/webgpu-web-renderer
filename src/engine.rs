use crate::bridge::{WebNativeBridge, LayoutRect, LayoutNode, Color};
use crate::dom::tree::DomTree;
use crate::dom::parser::HtmlParser;
use crate::css::parser::StyleSheet;
use crate::css::matcher::StyleMatcher;
use crate::layout::{LayoutItem, LayoutEnv, LayoutConverter, CpuLayoutCompute};
use crate::renderer::pipeline::RenderPipelineWrapper;
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
    click_handlers: HashMap<usize, Vec<ClickHandler>>,
    form_handlers: HashMap<String, FormHandler>,
    window_open_handlers: Vec<WindowOpenHandler>,
    current_url: String,
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

    /// 解析 CSS 并更新样式表
    fn parse_css(&mut self, css_text: &str) {
        self.stylesheet = StyleSheet::parse(css_text);
        self.style_matcher = Some(StyleMatcher::new(self.stylesheet.clone()));
        // 重新计算布局
        self.update_layout();
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
                    self.layout_items = items;

                    // 执行 CPU 布局计算
                    CpuLayoutCompute::compute(&mut self.layout_items, &env);

                    log::info!("Layout computed: {} items", self.layout_items.len());
                }
                Err(e) => {
                    log::error!("Layout conversion failed: {}", e);
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
            layout_items: Vec::new(),
            render_pipeline: None,
            click_handlers: HashMap::new(),
            form_handlers: HashMap::new(),
            window_open_handlers: Vec::new(),
            current_url: String::new(),
        }
    }

    /// 设置页面 HTML
    fn set_html(&mut self, html: &str) {
        let mut parser = HtmlParser::new(html);
        match parser.parse() {
            Ok(tree) => {
                self.dom_tree = tree;
                log::info!("HTML parsed successfully, {} nodes", self.dom_tree.len());
                // 触发布局更新
                self.update_layout();
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
        // 查找匹配的 DOM 节点
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
                // 触发布局更新
                self.update_layout();
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

        // 初始化渲染管线（首次调用时）
        if self.render_pipeline.is_none() {
            let mut pipeline = RenderPipelineWrapper::new(device.clone(), queue.clone());
            if let Err(e) = pipeline.initialize(self.width, self.height) {
                log::error!("Pipeline initialization failed: {}", e);
                return Vec::new();
            }
            self.render_pipeline = Some(pipeline);
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
        pipeline.update_uniforms(self.width, self.height);

        // 使用渲染管线绘制布局项
        if let Err(e) = pipeline.render(&texture_view, &self.layout_items) {
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
            if let Some(form_id) = self.query(form_selector) {
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
        self.width = width;
        self.height = height;
        // 重新计算布局
        self.update_layout();
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