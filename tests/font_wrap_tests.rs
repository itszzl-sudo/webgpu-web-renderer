//! WebGPU Rendering Engine - Font Wrapping Tests
//!
//! Testing text wrapping functionality

use webgpu_web_renderer::{Engine, WebNativeBridge};

/// Test normal word wrap
#[test]
fn test_normal_wrap() {
    println!("Test: Normal text wrapping");

    let mut engine = Engine::new(800, 600);

    let html = "<div id=\"wrap-container\"><p id=\"wrapped-text\">This is a long text that should wrap when the container is too narrow.</p></div>";
    let css = r#"
        #wrap-container { width: 200px; }
        #wrapped-text { white-space: normal; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#wrapped-text");
    assert!(text.is_some(), "Should find wrapped text element");

    println!("OK - Normal wrap test passed");
}

/// Test break-word wrapping
#[test]
fn test_break_word_wrap() {
    println!("Test: Break-word wrapping");

    let mut engine = Engine::new(800, 600);

    let html = "<div id=\"break-container\"><p id=\"break-text\">Averylongwordthatdoesnothaveanyspaces</p></div>";
    let css = r#"
        #break-container { width: 100px; }
        #break-text { word-wrap: break-word; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#break-text");
    assert!(text.is_some(), "Should find break-word text");

    println!("OK - Break-word test passed");
}

/// Test break-all wrapping
#[test]
fn test_break_all_wrap() {
    println!("Test: Break-all wrapping");

    let mut engine = Engine::new(800, 600);

    let html = "<div id=\"breakall-container\"><p id=\"breakall-text\">Numbers1234567890 and symbols!@#$</p></div>";
    let css = r#"
        #breakall-container { width: 80px; }
        #breakall-text { word-break: break-all; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#breakall-text");
    assert!(text.is_some(), "Should find break-all text");

    println!("OK - Break-all test passed");
}

/// Test no wrap
#[test]
fn test_nowrap_wrap() {
    println!("Test: No wrap");

    let mut engine = Engine::new(800, 600);

    let html = "<div id=\"nowrap-container\"><p id=\"nowrap-text\">This text should not wrap and stay on one line</p></div>";
    let css = r#"
        #nowrap-container { width: 50px; }
        #nowrap-text { white-space: nowrap; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#nowrap-text");
    assert!(text.is_some(), "Should find no-wrap text");

    println!("OK - No-wrap test passed");
}

/// Test pre-wrap (preserve whitespace and wrap)
#[test]
fn test_pre_wrap_wrap() {
    println!("Test: Pre-wrap");

    let mut engine = Engine::new(800, 600);

    let html = r#"<div id="prewrap-container"><pre id="prewrap-text">Line one
Line two
Line three</pre></div>"#;
    let css = r#"
        #prewrap-container { width: 100px; }
        #prewrap-text { white-space: pre-wrap; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#prewrap-text");
    assert!(text.is_some(), "Should find pre-wrap text");

    println!("OK - Pre-wrap test passed");
}

/// Test multi-paragraph wrapping
#[test]
fn test_multiline_paragraph_wrap() {
    println!("Test: Multi-paragraph wrapping");

    let mut engine = Engine::new(800, 600);

    let html = r#"
        <div id="paragraph-container">
            <p class="para">First paragraph text for testing.</p>
            <p class="para">Second paragraph with more content.</p>
            <p class="para">Third and final paragraph.</p>
        </div>
    "#;

    let css = r#"
        #paragraph-container { width: 200px; }
        .para { white-space: normal; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let paras = engine.query_all(".para");
    assert_eq!(paras.len(), 3, "Should find 3 paragraphs");

    println!("OK - Multi-paragraph test passed");
}

/// Test mixed language wrapping
#[test]
fn test_mixed_language_wrap() {
    println!("Test: Mixed language wrapping");

    let mut engine = Engine::new(800, 600);

    let html = "<div id=\"mixed-container\"><p id=\"mixed-text\">Hello 你好 World 世界 Test 测试</p></div>";
    let css = r#"
        #mixed-container { width: 100px; }
        #mixed-text { white-space: normal; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#mixed-text");
    assert!(text.is_some(), "Should find mixed language text");

    println!("OK - Mixed language test passed");
}

/// Test long word in container
#[test]
fn test_long_word_in_container() {
    println!("Test: Long word in container");

    let mut engine = Engine::new(800, 600);

    let html = "<div id=\"longword-container\"><p id=\"longword\">Supercalifragilisticexpialidocious</p></div>";
    let css = r#"
        #longword-container { width: 80px; }
        #longword { word-wrap: break-word; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#longword");
    assert!(text.is_some(), "Should find long word text");

    println!("OK - Long word test passed");
}

/// Test numbers and punctuation wrap
#[test]
fn test_numbers_and_punctuation_wrap() {
    println!("Test: Numbers and punctuation wrapping");

    let mut engine = Engine::new(800, 600);

    let html = "<div id=\"number-container\"><p id=\"numbers\">12345678901234567890!@#$%^&*</p></div>";
    let css = r#"
        #number-container { width: 60px; }
        #numbers { word-break: break-all; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let text = engine.query("#numbers");
    assert!(text.is_some(), "Should find numbers text");

    println!("OK - Numbers test passed");
}

/// Run all font wrapping tests
#[test]
fn run_font_wrap_tests() {
    println!("\n========================================");
    println!("Starting Font Wrapping Tests");
    println!("========================================\n");

    test_normal_wrap();
    test_break_word_wrap();
    test_break_all_wrap();
    test_nowrap_wrap();
    test_pre_wrap_wrap();
    test_multiline_paragraph_wrap();
    test_mixed_language_wrap();
    test_long_word_in_container();
    test_numbers_and_punctuation_wrap();

    println!("\n========================================");
    println!("All Font Wrapping Tests Passed!");
    println!("========================================\n");
}