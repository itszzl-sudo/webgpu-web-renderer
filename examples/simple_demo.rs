use webgpu_web_renderer::{Engine, WebNativeBridge};

fn main() {
    println!("WebGPU 渲染引擎 MVP 演示");
    
    let mut engine = Engine::new(800, 600);
    println!("引擎已创建");
    
    let html = r#"<div class="container"><h1>测试</h1></div>"#;
    engine.set_html(html);
    println!("HTML 已加载");
    
    let css = r#".container { padding: 10px; } h1 { color: red; }"#;
    engine.set_css(css);
    println!("CSS 已加载");
    
    if let Some(container_id) = engine.query(".container") {
        println!("找到容器节点 ID: {}", container_id);
    }
    
    let h1_id = engine.query("h1");
    println!("H1 节点: {:?}", h1_id);
    
    let all_divs = engine.query_all("div");
    println!("所有 div 节点: {:?}", all_divs);
    
    engine.set_style("h1", "color", "blue");
    println!("样式已更新");
    
    let render_result = engine.render();
    println!("渲染结果: {} 字节", render_result.len());
    
    println!("MVP 演示完成");
}