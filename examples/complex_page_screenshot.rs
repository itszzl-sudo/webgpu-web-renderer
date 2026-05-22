use webgpu_web_renderer::{Engine, WebNativeBridge};
use std::fs;

fn main() {
    println!("=== WebGPU 渲染引擎 - 复杂页面渲染截图 ===\n");

    let mut engine = Engine::new(1280, 900);

    let html = r##"
<div class="page">
    <header class="header">
        <div class="logo">MyApp</div>
        <nav class="nav">
            <a href="/">Home</a>
            <a href="/features">Features</a>
            <a href="/pricing">Pricing</a>
            <a href="/contact">Contact</a>
        </nav>
    </header>

    <section class="hero">
        <h1>WebGPU Render Engine</h1>
        <p class="subtitle">A pure Rust CSS layout engine powered by WebGPU compute and rendering</p>
        <div class="cta-buttons">
            <button class="btn btn-primary">Get Started</button>
            <button class="btn btn-secondary">Learn More</button>
        </div>
    </section>

    <section class="features">
        <div class="feature-card">
            <div class="icon-box" style="background:#4A90D9;">GPU</div>
            <h3>GPU Accelerated</h3>
            <p>Parallel compute shader for layout calculation</p>
        </div>
        <div class="feature-card">
            <div class="icon-box" style="background:#7B68EE;">BOX</div>
            <h3>Flexbox Layout</h3>
            <p>Full flex container with grow, shrink, align, justify</p>
        </div>
        <div class="feature-card">
            <div class="icon-box" style="background:#2ECC71;">CSS</div>
            <h3>CSS Styling</h3>
            <p>50+ CSS properties including shadows, transforms, borders</p>
        </div>
    </section>

    <section class="demo-panels">
        <div class="panel panel-left">
            <h2>Absolute Positioning</h2>
            <div class="abs-container">
                <div class="abs-box" style="background:#E74C3C;top:10px;left:10px;">TL</div>
                <div class="abs-box" style="background:#3498DB;top:10px;right:10px;">TR</div>
                <div class="abs-box" style="background:#2ECC71;bottom:10px;left:10px;">BL</div>
                <div class="abs-box" style="background:#F39C12;bottom:10px;right:10px;">BR</div>
                <div class="abs-center">Center Area</div>
            </div>
        </div>
        <div class="panel panel-right">
            <h2>Float and Clear</h2>
            <div class="float-container">
                <div class="float-box" style="float:left;background:#E74C3C;">Float L</div>
                <div class="float-box" style="float:right;background:#3498DB;">Float R</div>
                <p style="clear:both;">This text clears both floats and sits below them.</p>
            </div>
        </div>
    </section>

    <section class="card-section">
        <div class="card" style="box-shadow:4px 4px 10px rgba(0,0,0,0.2);border-radius:12px;">
            <h3>Card with Shadow</h3>
            <p>box-shadow and border-radius applied.</p>
        </div>
        <div class="card" style="border:3px solid #E74C3C;border-radius:50%;width:120px;height:120px;text-align:center;">
            <h3 style="margin-top:35px;">Circle</h3>
        </div>
        <div class="card" style="background:#667eea;">
            <h3 style="color:white;">Accent Card</h3>
            <p style="color:rgba(255,255,255,0.8);">Background with accent color.</p>
        </div>
    </section>

    <section class="overflow-demo">
        <h2>Scroll Container</h2>
        <div class="scroll-box" style="overflow:scroll;width:100%;height:150px;border:1px solid #ccc;">
            <div style="width:200%;height:200px;background:linear-gradient(90deg,#ff6b6b,#ffd93d,#6bcb77,#4d96ff);">
                <p style="padding:20px;color:white;font-weight:bold;">
                    Scroll horizontally and vertically to see overflow content in action!
                </p>
            </div>
        </div>
    </section>

    <footer class="footer">
        <p>Copyright 2026 WebGPU Web Renderer. Built with Rust and WebGPU.</p>
    </footer>
</div>
"##;

    let css = r##"
body { margin:0; padding:0; background:#f0f2f5; }
.page { max-width:1100px; margin:0 auto; background:white; min-height:900px; }

.header { display:flex; align-items:center; justify-content:space-between;
          padding:16px 32px; background:#2c3e50; color:white; }
.logo { font-size:24px; font-weight:bold; }
.nav { display:flex; gap:20px; }
.nav a { color:rgba(255,255,255,0.8); text-decoration:none; font-size:14px; }

.hero { text-align:center; padding:60px 20px; background:#667eea; color:white; }
.hero h1 { font-size:36px; margin:0 0 12px; }
.hero .subtitle { font-size:18px; opacity:0.9; margin:0 0 30px; }
.cta-buttons { display:flex; justify-content:center; gap:16px; }
.btn { padding:12px 28px; border-radius:6px; border:none; font-size:16px; }
.btn-primary { background:white; color:#667eea; font-weight:bold; }
.btn-secondary { background:transparent; color:white; border:2px solid rgba(255,255,255,0.5); }

.features { display:flex; justify-content:center; gap:24px; padding:48px 20px; flex-wrap:wrap; }
.feature-card { width:280px; padding:24px; border-radius:12px; background:white;
                box-shadow:0 2px 12px rgba(0,0,0,0.08); text-align:center; }
.feature-card h3 { margin:12px 0 8px; font-size:18px; }
.feature-card p { font-size:14px; color:#666; margin:0; }
.icon-box { width:48px; height:48px; border-radius:12px; display:inline-flex;
            align-items:center; justify-content:center; font-size:18px; color:white; }

.demo-panels { display:flex; gap:24px; padding:32px 20px; }
.panel { flex:1; padding:20px; border-radius:10px; background:#f8f9fa; }
.panel h2 { margin:0 0 16px; font-size:18px; }

.abs-container { position:relative; height:140px; background:#e9ecef; border-radius:8px; }
.abs-box { position:absolute; width:50px; height:30px; border-radius:4px;
           color:white; font-size:11px; display:flex; align-items:center; justify-content:center; }
.abs-center { position:absolute; top:50%; left:50%; transform:translate(-50%,-50%); font-size:12px; color:#666; }

.float-container { min-height:80px; }
.float-box { width:80px; height:36px; margin:4px; border-radius:4px;
             color:white; font-size:12px; display:flex; align-items:center; justify-content:center; }

.card-section { display:flex; gap:20px; padding:20px; flex-wrap:wrap; }
.card { flex:1; min-width:180px; padding:20px; background:white; border-radius:8px; box-shadow:0 1px 6px rgba(0,0,0,0.06); }
.card h3 { margin:0 0 8px; font-size:16px; }
.card p { margin:0; font-size:13px; color:#666; }

.overflow-demo { padding:20px; }
.overflow-demo h2 { margin:0 0 12px; font-size:18px; }
.scroll-box { border-radius:8px; }

.footer { text-align:center; padding:20px; background:#2c3e50; color:rgba(255,255,255,0.7); font-size:13px; }
.footer p { margin:0; }
"##;

    // 加载 HTML 和 CSS
    engine.set_html(html);
    engine.set_css(css);

    // 强制提交待处理的变更
    engine.flush();

    // 查询元素并输出统计
    println!("页面元素统计:");
    println!("  div 元素: {:?}", engine.query_all("div").len());
    println!("  a 链接: {:?}", engine.query_all("a").len());
    println!("  h1-h3: {:?}", engine.query_all("h1").len() + engine.query_all("h2").len() + engine.query_all("h3").len());
    println!("  button: {:?}", engine.query_all("button").len());

    // 渲染
    println!("\n正在渲染 (1280x900)...");
    let png_data = engine.render();
    println!("渲染完成: {} 字节", png_data.len());

    if png_data.is_empty() {
        eprintln!("错误: 渲染结果为空 (可能需要 WebGPU 设备)");
        std::process::exit(1);
    }

    // 保存 PNG
    let output_path = "screenshot_complex_page.png";
    fs::write(output_path, &png_data)
        .expect("无法写入截图文件");
    println!("\n截图已保存: {} ({} KB)", output_path, png_data.len() / 1024);
    println!("=== 完成 ===");
}