use webgpu_web_renderer::{Engine, WebNativeBridge};
use std::fs;

fn main() {
    println!("=== Cool Effects HTML 渲染截图 ===\n");

    let mut engine = Engine::new(1280, 900);

    // 读取 cool-effects.html 文件
    let html_path = "examples/cool-effects.html";
    let full_html = fs::read_to_string(html_path)
        .expect("Failed to read examples/cool-effects.html");

    // 提取 body 内的 HTML 元素（跳过 DOCTYPE/head/script）
    let body_html = extract_body_content(&full_html);

    // 使用简化的 CSS（去除引擎不支持的属性）
    let css = r##"
body { margin:0; padding:0; background:#0a0e27; width:1280px; height:900px; position:relative; }

/* Corner Shapes */
.corner-shape { position:fixed; width:200px; height:200px; }
.top-left { top:0; left:0; background:rgba(255,110,199,0.3); }
.top-right { top:0; right:0; background:rgba(124,58,237,0.3); }
.bottom-left { bottom:0; left:0; background:rgba(59,130,246,0.3); }
.bottom-right { bottom:0; right:0; background:rgba(100,255,218,0.3); }

/* Floating Hexagons */
.hex-container { position:fixed; top:0; left:0; width:100%; height:100%; pointer-events:none; }
.hexagon { position:absolute; width:80px; height:46px; background:rgba(255,110,199,0.2); border:2px solid rgba(100,255,218,0.4); }
.hex-1 { top:15%; left:10%; }
.hex-2 { top:25%; right:15%; background:rgba(124,58,237,0.2); }
.hex-3 { bottom:20%; left:20%; }
.hex-4 { bottom:15%; right:10%; background:rgba(59,130,246,0.2); }

/* Glow Orbs */
.orb { position:fixed; border-radius:50%; pointer-events:none; }
.orb-1 { width:350px; height:350px; background:rgba(255,110,199,0.2); top:20%; left:20%; }
.orb-2 { width:300px; height:300px; background:rgba(124,58,237,0.2); bottom:25%; right:15%; }
.orb-3 { width:250px; height:250px; background:rgba(100,255,218,0.2); top:60%; left:70%; }

/* Main Scene */
.scene { position:absolute; top:50%; left:50%; width:600px; height:600px; }

/* Rings */
.ring { position:absolute; top:50%; left:50%; border:3px solid; border-radius:50%; }
.ring-1 { width:350px; height:350px; border-color:rgba(255,110,199,0.6); }
.ring-2 { width:420px; height:420px; border-color:rgba(124,58,237,0.5); }
.ring-3 { width:500px; height:500px; border-color:rgba(59,130,246,0.4); }

/* Sphere */
.sphere { position:absolute; top:50%; left:50%; width:200px; height:200px;
          background:rgba(124,58,237,0.8); border-radius:50%;
          box-shadow:0 0 60px rgba(124,58,237,0.6); }

/* Gems */
.gem { position:absolute; width:24px; height:24px; background:rgba(100,255,218,0.9);
       box-shadow:0 0 20px rgba(100,255,218,0.8); }
.gem-1 { top:50%; left:50%; }
.gem-2 { top:50%; left:50%; }
.gem-3 { top:50%; left:50%; }

/* Particles */
.particle { position:absolute; border-radius:50%; }
"##;

    // 加载 HTML 和 CSS
    println!("HTML 长度: {} 字符", body_html.len());
    println!("CSS 长度: {} 字符", css.len());

    engine.set_html(&body_html);
    engine.set_css(css);
    engine.flush();

    // 元素统计
    println!("\n渲染元素统计:");
    println!("  div: {:?}", engine.query_all("div").len());
    println!("  body: {:?}", engine.query_all("body").len());

    // 渲染
    println!("\n正在渲染 (1280x900)...");
    let png_data = engine.render();
    println!("渲染完成: {} 字节", png_data.len());

    if png_data.is_empty() {
        eprintln!("错误: 渲染结果为空 (需要 WebGPU 设备)");
        std::process::exit(1);
    }

    // 保存 PNG
    let output_path = "screenshot_cool_effects.png";
    fs::write(output_path, &png_data)
        .expect("无法写入截图文件");
    println!("\n截图已保存: {} ({} KB)", output_path, png_data.len() / 1024);
    println!("=== 完成 ===");
}

/// 从完整 HTML 文件中提取 body 内的元素内容
fn extract_body_content(full_html: &str) -> String {
    let mut result = String::new();

    // 移除所有 script 块
    let mut no_script = String::new();
    let mut in_script = false;
    for line in full_html.lines() {
        if line.trim().starts_with("<script") || line.trim().starts_with("<Script") {
            in_script = true;
        }
        if !in_script {
            no_script.push_str(line);
            no_script.push('\n');
        }
        if line.trim().starts_with("</script") || line.trim().starts_with("</Script") {
            in_script = false;
        }
    }

    // 提取 <body> 和 </body> 之间的内容
    let body_start = no_script.find("<body").and_then(|i| {
        no_script[i..].find('>').map(|j| i + j + 1)
    });

    let body_end = no_script.rfind("</body>");

    if let (Some(start), Some(end)) = (body_start, body_end) {
        let inner = &no_script[start..end];
        // 只保留 div 元素（去掉 svg, style 等）
        for line in inner.lines() {
            let trimmed = line.trim();
            // 跳过 SVG、style、meta 等
            if trimmed.starts_with("<svg") || trimmed.starts_with("<style") {
                continue;
            }
            if trimmed.starts_with("</svg") || trimmed.starts_with("</style") {
                continue;
            }
            if trimmed.starts_with("<!--") {
                continue;
            }
            // 只保留 div 元素
            if trimmed.starts_with("<div") || trimmed.starts_with("</div") || trimmed.starts_with("<body") {
                result.push_str(line);
                result.push('\n');
            }
        }
    } else {
        // 如果没有 body 标签，直接用原始 HTML
        result = no_script;
    }

    result
}