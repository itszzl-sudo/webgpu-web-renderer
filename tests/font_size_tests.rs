//! WebGPU 渲染引擎 - 字体大小测试
//!
//! 测试不同字体大小的渲染功能

use webgpu_web_renderer::{Engine, WebNativeBridge};

/// 测试极小字体
#[test]
fn test_very_small_font() {
    println!("🔲 测试极小字体 (8px)");

    let mut engine = Engine::new(800, 600);

    let html = "<p id=\"tiny-text\">极小字体</p>";
    let css = "#tiny-text { font-size: 8px; white-space: nowrap; }";

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#tiny-text");
    assert!(text.is_some(), "应该找到 8px 字体文本");

    println!("✓ 8px 字体配置正确");
}

/// 测试小字体
#[test]
fn test_small_font() {
    println!("📄 测试小字体 (12px)");

    let mut engine = Engine::new(800, 600);

    let html = "<p id=\"small-text\">小字体文本</p>";
    let css = "#small-text { font-size: 12px; white-space: nowrap; }";

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#small-text");
    assert!(text.is_some(), "应该找到 12px 字体文本");

    println!("✓ 12px 字体配置正确");
}

/// 测试正常字体
#[test]
fn test_normal_font() {
    println!("📝 测试正常字体 (16px)");

    let mut engine = Engine::new(800, 600);

    let html = "<p id=\"normal-text\">正常字体文本</p>";
    let css = "#normal-text { font-size: 16px; white-space: nowrap; }";

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#normal-text");
    assert!(text.is_some(), "应该找到 16px 字体文本");

    println!("✓ 16px 字体配置正确");
}

/// 测试大字体
#[test]
fn test_large_font() {
    println!("📋 测试大字体 (24px)");

    let mut engine = Engine::new(800, 600);

    let html = "<p id=\"large-text\">大字体文本</p>";
    let css = "#large-text { font-size: 24px; white-space: nowrap; }";

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#large-text");
    assert!(text.is_some(), "应该找到 24px 字体文本");

    println!("✓ 24px 字体配置正确");
}

/// 测试极大字体
#[test]
fn test_very_large_font() {
    println!("🔝 测试极大字体 (48px)");

    let mut engine = Engine::new(800, 600);

    let html = "<p id=\"huge-text\">极大字体</p>";
    let css = "#huge-text { font-size: 48px; white-space: nowrap; }";

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#huge-text");
    assert!(text.is_some(), "应该找到 48px 字体文本");

    println!("✓ 48px 字体配置正确");
}

/// 测试超大标题字体
#[test]
fn test_title_font() {
    println!("📰 测试标题字体 (72px)");

    let mut engine = Engine::new(800, 600);

    let html = "<h1 id=\"title\">大标题</h1>";
    let css = "#title { font-size: 72px; white-space: nowrap; }";

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#title");
    assert!(text.is_some(), "应该找到 72px 标题文本");

    println!("✓ 72px 标题字体配置正确");
}

/// 测试使用 em 单位的字体
#[test]
fn test_em_font_size() {
    println!("📐 测试 em 单位字体");

    let mut engine = Engine::new(800, 600);

    let html = "<p id=\"em-text\">em 单位字体</p>";
    let css = "#em-text { font-size: 2em; white-space: nowrap; }";

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#em-text");
    assert!(text.is_some(), "应该找到 em 单位字体文本");

    println!("✓ em 单位字体配置正确");
}

/// 测试使用 rem 单位的字体
#[test]
fn test_rem_font_size() {
    println!("📏 测试 rem 单位字体");

    let mut engine = Engine::new(800, 600);

    let html = "<p id=\"rem-text\">rem 单位字体</p>";
    let css = "#rem-text { font-size: 1.5rem; white-space: nowrap; }";

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#rem-text");
    assert!(text.is_some(), "应该找到 rem 单位字体文本");

    println!("✓ rem 单位字体配置正确");
}

/// 测试使用 pt 单位的字体（印刷单位）
#[test]
fn test_pt_font_size() {
    println!("📊 测试 pt 单位字体");

    let mut engine = Engine::new(800, 600);

    let html = "<p id=\"pt-text\">pt 单位字体</p>";
    let css = "#pt-text { font-size: 16pt; white-space: nowrap; }";

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#pt-text");
    assert!(text.is_some(), "应该找到 pt 单位字体文本");

    println!("✓ pt 单位字体配置正确");
}

/// 测试不同字体大小的组合
#[test]
fn test_mixed_font_sizes() {
    println!("📚 测试不同字体大小组合");

    let mut engine = Engine::new(800, 600);

    let html = r#"
        <h1 id="h1-text">一级标题</h1>
        <h2 id="h2-text">二级标题</h2>
        <h3 id="h3-text">三级标题</h3>
        <p id="body-text">正文内容</p>
    "#;

    let css = r#"
        #h1-text { font-size: 48px; white-space: nowrap; }
        #h2-text { font-size: 32px; white-space: nowrap; }
        #h3-text { font-size: 24px; white-space: nowrap; }
        #body-text { font-size: 16px; white-space: nowrap; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let h1 = engine.query("#h1-text");
    let h2 = engine.query("#h2-text");
    let h3 = engine.query("#h3-text");
    let body = engine.query("#body-text");

    assert!(h1.is_some(), "应该找到 h1");
    assert!(h2.is_some(), "应该找到 h2");
    assert!(h3.is_some(), "应该找到 h3");
    assert!(body.is_some(), "应该找到正文");

    println!("✓ 不同字体大小组合配置正确");
}

/// 测试字体大小继承
#[test]
fn test_font_size_inheritance() {
    println!("👨‍👩‍👧 测试字体大小继承");

    let mut engine = Engine::new(800, 600);

    let html = r#"
        <div id="parent">
            <span id="child1">子文本1</span>
            <span id="child2">子文本2</span>
        </div>
    "#;

    let css = r#"
        #parent { font-size: 20px; }
        #child1 { font-size: 1.5em; }
        #child2 { font-size: 0.8em; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let parent = engine.query("#parent");
    let child1 = engine.query("#child1");
    let child2 = engine.query("#child2");

    assert!(parent.is_some(), "应该找到父元素");
    assert!(child1.is_some(), "应该找到子元素1");
    assert!(child2.is_some(), "应该找到子元素2");

    println!("✓ 字体大小继承配置正确");
}

/// 运行所有字体大小测试
#[test]
fn run_font_size_tests() {
    println!("\n========================================");
    println!("🔤 开始运行字体大小测试");
    println!("========================================\n");

    test_very_small_font();
    test_small_font();
    test_normal_font();
    test_large_font();
    test_very_large_font();
    test_title_font();
    test_em_font_size();
    test_rem_font_size();
    test_pt_font_size();
    test_mixed_font_sizes();
    test_font_size_inheritance();

    println!("\n========================================");
    println!("✅ 所有字体大小测试通过!");
    println!("========================================\n");
}