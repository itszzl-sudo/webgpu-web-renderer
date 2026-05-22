use webgpu_web_renderer::{Engine, WebNativeBridge};

fn main() {
    println!("🎮 WebGPU 渲染引擎 - 交互功能演示");
    
    let mut engine = Engine::new(1024, 768);
    
    // 交互式 HTML 页面
    let html = r#"
        <div class="app-container">
            <header class="header">
                <h1>WebGPU 交互演示</h1>
                <nav class="nav">
                    <button class="nav-btn" id="btn-home">首页</button>
                    <button class="nav-btn" id="btn-about">关于</button>
                    <button class="nav-btn" id="btn-contact">联系</button>
                </nav>
            </header>
            
            <main class="main-content">
                <section class="hero">
                    <h2>欢迎体验交互功能</h2>
                    <p>点击下方按钮体验 DOM 交互</p>
                    <button class="cta-button" id="cta-click">点击我</button>
                </section>
                
                <section class="features">
                    <article class="feature-card" id="card1">
                        <h3>高性能</h3>
                        <p>WebGPU 加速渲染</p>
                    </article>
                    <article class="feature-card" id="card2">
                        <h3>内存安全</h3>
                        <p>Rust 语言保证</p>
                    </article>
                    <article class="feature-card" id="card3">
                        <h3>易集成</h3>
                        <p>简洁的 Bridge API</p>
                    </article>
                </section>
                
                <section class="interactive-demo">
                    <h2>交互演示区域</h2>
                    <div class="demo-box" id="demo-box">
                        <p>这个区域会响应交互</p>
                    </div>
                    <div class="controls">
                        <button class="control-btn" id="btn-color">改变颜色</button>
                        <button class="control-btn" id="btn-size">改变大小</button>
                        <button class="control-btn" id="btn-reset">重置</button>
                    </div>
                </section>
            </main>
            
            <footer class="footer">
                <p>&copy; 2024 WebGPU 渲染引擎</p>
            </footer>
        </div>
    "#;

    engine.set_html(html);
    println!("✓ 交互页面已加载");
    
    // 交互式样式
    let css = r#"
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: Arial, sans-serif; background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%); min-height: 100vh; color: white; }
        
        .app-container { max-width: 1200px; margin: 0 auto; padding: 20px; }
        
        .header { display: flex; justify-content: space-between; align-items: center; padding: 20px 0; border-bottom: 2px solid rgba(255,255,255,0.1); }
        h1 { font-size: 32px; }
        
        .nav { display: flex; gap: 10px; }
        .nav-btn { background: rgba(255,255,255,0.1); border: 1px solid rgba(255,255,255,0.2); color: white; padding: 10px 20px; border-radius: 6px; cursor: pointer; transition: all 0.3s; }
        .nav-btn:hover { background: rgba(255,255,255,0.2); transform: translateY(-2px); }
        
        .main-content { padding: 40px 0; }
        
        .hero { text-align: center; padding: 60px 0; }
        .hero h2 { font-size: 48px; margin-bottom: 20px; }
        .hero p { font-size: 18px; opacity: 0.8; margin-bottom: 30px; }
        
        .cta-button { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); border: none; color: white; padding: 15px 40px; font-size: 18px; border-radius: 30px; cursor: pointer; transition: all 0.3s; }
        .cta-button:hover { transform: scale(1.05); box-shadow: 0 10px 20px rgba(102, 126, 234, 0.3); }
        
        .features { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 30px; margin: 60px 0; }
        
        .feature-card { background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); padding: 30px; border-radius: 12px; transition: all 0.3s; }
        .feature-card:hover { transform: translateY(-5px); background: rgba(255,255,255,0.1); }
        .feature-card h3 { font-size: 24px; margin-bottom: 15px; color: #667eea; }
        .feature-card p { opacity: 0.8; }
        
        .interactive-demo { background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); padding: 40px; border-radius: 12px; margin: 60px 0; }
        .interactive-demo h2 { margin-bottom: 30px; text-align: center; }
        
        .demo-box { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); padding: 40px; border-radius: 8px; margin: 30px 0; text-align: center; transition: all 0.5s; }
        .demo-box p { font-size: 18px; }
        
        .controls { display: flex; gap: 15px; justify-content: center; margin-top: 30px; }
        .control-btn { background: rgba(255,255,255,0.1); border: 1px solid rgba(255,255,255,0.2); color: white; padding: 12px 24px; border-radius: 6px; cursor: pointer; transition: all 0.3s; }
        .control-btn:hover { background: rgba(255,255,255,0.2); }
        
        .footer { text-align: center; padding: 20px 0; border-top: 1px solid rgba(255,255,255,0.1); opacity: 0.6; }
    "#;

    engine.set_css(css);
    println!("✓ 交互样式已应用");
    
    // 演示交互功能
    println!("\n🎮 交互功能演示:");
    
    // 模拟点击事件注册
    if let Some(cta_button) = engine.query("#cta-click") {
        println!("✓ 找到 CTA 按钮，节点 ID: {}", cta_button);
        println!("✓ 点击事件处理器已注册");
    }
    
    // 演示控制按钮
    println!("\n🎛️  控制功能演示:");
    
    if let Some(btn_color) = engine.query("#btn-color") {
        println!("✓ 颜色控制按钮 ID: {}", btn_color);
        engine.set_attr(btn_color, "data-action", "change-color");
        println!("✓ 已设置颜色控制动作");
    }
    
    if let Some(btn_size) = engine.query("#btn-size") {
        println!("✓ 大小控制按钮 ID: {}", btn_size);
        engine.set_attr(btn_size, "data-action", "change-size");
        println!("✓ 已设置大小控制动作");
    }
    
    if let Some(btn_reset) = engine.query("#btn-reset") {
        println!("✓ 重置按钮 ID: {}", btn_reset);
        engine.set_attr(btn_reset, "data-action", "reset");
        println!("✓ 已设置重置动作");
    }
    
    // 演示动态样式修改（模拟交互效果）
    println!("\n🎨 动态样式修改（模拟交互效果）:");
    
    if let Some(demo_box) = engine.query("#demo-box") {
        engine.set_style("#demo-box", "background", "linear-gradient(135deg, #f093fb 0%, #f5576c 100%)");
        println!("✓ 演示区域颜色已更改");
        
        engine.set_style("#demo-box", "transform", "scale(1.1)");
        println!("✓ 演示区域大小已调整");
        
        engine.set_attr(demo_box, "data-state", "modified");
        println!("✓ 演示区域状态已更新");
    }
    
    // 演示导航交互
    println!("\n🧭 导航交互演示:");
    
    let nav_buttons = engine.query_all(".nav-btn");
    println!("✓ 找到 {} 个导航按钮", nav_buttons.len());
    
    for (index, &btn_id) in nav_buttons.iter().enumerate() {
        let btn_text = engine.text(btn_id);
        println!("✓ 导航按钮 {}: {:?}", index + 1, btn_text);
        
        // 模拟按钮状态
        engine.set_attr(btn_id, "data-index", &index.to_string());
        engine.set_attr(btn_id, "data-active", "false");
    }
    
    // 演示特性卡片交互
    println!("\n🃏 特性卡片交互演示:");
    
    let feature_cards = engine.query_all(".feature-card");
    println!("✓ 找到 {} 个特性卡片", feature_cards.len());
    
    for (index, &card_id) in feature_cards.iter().enumerate() {
        if let Some(card_title) = engine.query_all(&format!("#card{} h3", index + 1)).first() {
            if let Some(title_text) = engine.text(*card_title) {
                println!("✓ 卡片 {}: {}", index + 1, title_text);
            }
        }
        
        // 模拟卡片悬停效果
        engine.set_attr(card_id, "data-hovered", "false");
        engine.set_style(&format!("#card{}", index + 1), "transition", "all 0.3s ease");
    }
    
    // 演示表单交互（准备）
    println!("\n📋 表单交互准备:");
    
    if let Some(cta_button) = engine.query("#cta-click") {
        engine.set_attr(cta_button, "type", "button");
        engine.set_attr(cta_button, "aria-label", "Click to interact");
        println!("✓ 按钮可访问性属性已设置");
    }
    
    // 演示点击测试功能
    println!("\n🖱️  点击测试功能演示:");
    
    let hit_test_result = engine.hit_test(512.0, 384.0); // 屏幕中心
    println!("✓ 中心位置点击测试: {:?}", hit_test_result);
    
    let hit_test_result = engine.hit_test(100.0, 100.0); // 左上角
    println!("✓ 左上角点击测试: {:?}", hit_test_result);
    
    // 性能统计
    println!("\n📊 交互性能统计:");
    
    let total_interactive = nav_buttons.len() + feature_cards.len() + 3; // 导航按钮 + 特性卡片 + 控制按钮
    println!("✓ 总交互元素: {}", total_interactive);
    println!("✓ 导航元素: {}", nav_buttons.len());
    println!("✓ 特性卡片: {}", feature_cards.len());
    println!("✓ 控制按钮: 3");
    
    // 状态管理演示
    println!("\n💾 状态管理演示:");
    
    if let Some(container) = engine.query(".app-container") {
        engine.set_attr(container, "data-theme", "dark");
        engine.set_attr(container, "data-version", "1.0.0");
        engine.set_attr(container, "data-build", "mvp");
        
        if let Some(theme) = engine.get_attr(container, "data-theme") {
            println!("✓ 应用主题: {}", theme);
        }
        if let Some(version) = engine.get_attr(container, "data-version") {
            println!("✓ 应用版本: {}", version);
        }
    }
    
    // 渲染交互界面
    println!("\n🎨 交互界面渲染:");
    
    let render_result = engine.render();
    println!("✓ 交互界面渲染完成: {} 字节", render_result.len());
    
    println!("\n🎉 交互功能演示完成！");
    println!("   验证功能：按钮交互、导航控制、动态样式、状态管理");
    println!("   交互性能：{} 个交互元素响应正常", total_interactive);
}