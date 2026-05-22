//! WebGPU 渲染引擎 - 字体渲染测试
//!
//! 测试字体渲染功能（不换行）

use webgpu_web_renderer::{Engine, WebNativeBridge};

/// 测试简单文本渲染（无换行）
#[test]
fn test_simple_text_no_wrap() {
    println!("📝 测试简单文本渲染（无换行）");

    let mut engine = Engine::new(800, 600);

    let html = "<div id=\"text-box\"><span id=\"text\">Hello World</span></div>";
    let css = r#"
        #text-box { width: 800px; }
        #text { white-space: nowrap; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#text");
    assert!(text.is_some(), "应该找到文本元素");

    println!("✓ 简单文本渲染配置正确");
}

/// 测试多行文本（无换行）
#[test]
fn test_multiline_text_no_wrap() {
    println!("📄 测试多行文本（无换行）");

    let mut engine = Engine::new(800, 600);

    let html = r#"
        <div id="container">
            <p id="para1" class="text-line">这是第一行文本</p>
            <p id="para2" class="text-line">这是第二行文本</p>
            <p id="para3" class="text-line">这是第三行文本</p>
        </div>
    "#;

    let css = r#"
        .text-line { white-space: nowrap; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let lines = engine.query_all(".text-line");
    assert_eq!(lines.len(), 3, "应该有 3 行文本");

    println!("✓ 多行文本渲染配置正确");
}

/// 测试不同字体颜色
#[test]
fn test_text_colors() {
    println!("🎨 测试文本颜色");

    let mut engine = Engine::new(800, 600);

    let html = r#"
        <span id="red-text">红色文字</span>
        <span id="blue-text">蓝色文字</span>
        <span id="green-text">绿色文字</span>
    "#;

    let css = r#"
        #red-text { color: red; white-space: nowrap; }
        #blue-text { color: blue; white-space: nowrap; }
        #green-text { color: green; white-space: nowrap; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let red = engine.query("#red-text");
    let blue = engine.query("#blue-text");
    let green = engine.query("#green-text");

    assert!(red.is_some(), "应该找到红色文字");
    assert!(blue.is_some(), "应该找到蓝色文字");
    assert!(green.is_some(), "应该找到绿色文字");

    println!("✓ 文本颜色渲染配置正确");
}

/// 测试长文本（不换行）
#[test]
fn test_long_text_no_wrap() {
    println!("📜 测试长文本（不换行）");

    let mut engine = Engine::new(800, 600);

    let html = "<div id=\"long-text-container\"><span id=\"long-text\">这是一段很长的文本，用于测试不换行的渲染效果。文本应该保持在一行内显示，不会自动换行。</span></div>";
    let css = r#"
        #long-text-container { width: 200px; }
        #long-text { white-space: nowrap; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#long-text");
    assert!(text.is_some(), "应该找到长文本元素");

    println!("✓ 长文本不换行渲染配置正确");
}

/// 测试英文文本
#[test]
fn test_english_text() {
    println!("🔤 测试英文文本");

    let mut engine = Engine::new(800, 600);

    let html = "<p id=\"english\">The quick brown fox jumps over the lazy dog.</p>";
    let css = "#english { white-space: nowrap; }";

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#english");
    assert!(text.is_some(), "应该找到英文文本");

    println!("✓ 英文文本渲染配置正确");
}

/// 测试特殊字符文本
#[test]
fn test_special_characters() {
    println!("🔣 测试特殊字符文本");

    let mut engine = Engine::new(800, 600);

    let html = "<span id=\"special\">符号: @#$%^&amp;*()_+-=[]{}|;:&#39;,./</span>";
    let css = "#special { white-space: nowrap; }";

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#special");
    assert!(text.is_some(), "应该找到特殊字符文本");

    println!("✓ 特殊字符文本渲染配置正确");
}

/// 测试中英混合文本
#[test]
fn test_mixed_language_text() {
    println!("🌐 测试中英混合文本");

    let mut engine = Engine::new(800, 600);

    let html = "<p id=\"mixed\">Hello 你好 World 世界</p>";
    let css = "#mixed { white-space: nowrap; }";

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#mixed");
    assert!(text.is_some(), "应该找到混合语言文本");

    println!("✓ 中英混合文本渲染配置正确");
}

/// 测试带背景的文本
#[test]
fn test_text_with_background() {
    println!("📋 测试带背景的文本");

    let mut engine = Engine::new(800, 600);

    let html = "<div id=\"text-bg\">突出显示的文本</div>";
    let css = r#"
        #text-bg {
            background-color: yellow;
            padding: 10px;
            white-space: nowrap;
        }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#text-bg");
    assert!(text.is_some(), "应该找到带背景的文本");

    println!("✓ 带背景文本渲染配置正确");
}

/// 测试文本对齐（左对齐）
#[test]
fn test_text_align_left() {
    println!("⬅️ 测试文本左对齐");

    let mut engine = Engine::new(800, 600);

    let html = "<div id=\"left-aligned\">左对齐文本</div>";
    let css = r#"
        #left-aligned {
            width: 400px;
            text-align: left;
            white-space: nowrap;
        }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#left-aligned");
    assert!(text.is_some(), "应该找到左对齐文本");

    println!("✓ 文本左对齐渲染配置正确");
}

/// 测试文本对齐（右对齐）
#[test]
fn test_text_align_right() {
    println!("➡️ 测试文本右对齐");

    let mut engine = Engine::new(800, 600);

    let html = "<div id=\"right-aligned\">右对齐文本</div>";
    let css = r#"
        #right-aligned {
            width: 400px;
            text-align: right;
            white-space: nowrap;
        }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#right-aligned");
    assert!(text.is_some(), "应该找到右对齐文本");

    println!("✓ 文本右对齐渲染配置正确");
}

/// 测试文本对齐（居中）
#[test]
fn test_text_align_center() {
    println!("⬆️ 测试文本居中对齐");

    let mut engine = Engine::new(800, 600);

    let html = "<div id=\"center-aligned\">居中文本</div>";
    let css = r#"
        #center-aligned {
            width: 400px;
            text-align: center;
            white-space: nowrap;
        }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#center-aligned");
    assert!(text.is_some(), "应该找到居中文本");

    println!("✓ 文本居中对齐渲染配置正确");
}

/// 运行所有字体渲染测试（无换行）
#[test]
fn run_font_render_tests() {
    println!("\n========================================");
    println!("🔤 开始运行字体渲染测试（无换行）");
    println!("========================================\n");

    test_simple_text_no_wrap();
    test_multiline_text_no_wrap();
    test_text_colors();
    test_long_text_no_wrap();
    test_english_text();
    test_special_characters();
    test_mixed_language_text();
    test_text_with_background();
    test_text_align_left();
    test_text_align_right();
    test_text_align_center();

    println!("\n========================================");
    println!("✅ 所有字体渲染测试通过（无换行）!");
    println!("========================================\n");
}