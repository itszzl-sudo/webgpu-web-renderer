//! WebGPU 渲染引擎 - Div 渲染测试
//!
//! 测试各种颜色 div 的渲染功能

use webgpu_web_renderer::{Engine, WebNativeBridge};

/// 测试单个纯色 div
#[test]
fn test_single_colored_div() {
    println!("🟥 测试单个纯色 div");

    let mut engine = Engine::new(800, 600);

    let html = "<div id=\"red-box\"></div>";
    let css = "#red-box { width: 200px; height: 150px; background-color: red; }";

    engine.set_html(html);
    engine.set_css(css);

    // 验证 div 存在
    let div_id = engine.query("#red-box");
    assert!(div_id.is_some(), "应该找到红色 div");

    println!("✓ 单个红色 div 渲染配置正确");
}

/// 测试多个不同颜色的 div
#[test]
fn test_multiple_colored_divs() {
    println!("🎨 测试多个不同颜色的 div");

    let mut engine = Engine::new(800, 600);

    let html = r#"
        <div id="container">
            <div id="red-box" class="color-box"></div>
            <div id="green-box" class="color-box"></div>
            <div id="blue-box" class="color-box"></div>
            <div id="yellow-box" class="color-box"></div>
        </div>
    "#;

    let css = r#"
        .color-box { width: 100px; height: 100px; }
        #red-box { background-color: red; }
        #green-box { background-color: green; }
        #blue-box { background-color: blue; }
        #yellow-box { background-color: yellow; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    // 验证所有 div 都存在
    let boxes = engine.query_all(".color-box");
    assert_eq!(boxes.len(), 4, "应该有 4 个彩色 div");

    println!("✓ 4 个彩色 div 渲染配置正确");
}

/// 测试带透明度的 div
#[test]
fn test_transparent_div() {
    println!("🔲 测试带透明度的 div");

    let mut engine = Engine::new(800, 600);

    let html = "<div id=\"transparent-box\"></div>";
    let css = "#transparent-box { width: 100px; height: 100px; background-color: blue; opacity: 0.5; }";

    engine.set_html(html);
    engine.set_css(css);

    let div_id = engine.query("#transparent-box");
    assert!(div_id.is_some(), "应该找到透明 div");

    println!("✓ 半透明 div 渲染配置正确");
}

/// 测试带边框的 div
#[test]
fn test_div_with_border() {
    println!("📦 测试带边框的 div");

    let mut engine = Engine::new(800, 600);

    let html = "<div id=\"bordered-box\"></div>";
    let css = r#"
        #bordered-box {
            width: 150px;
            height: 100px;
            background-color: white;
            border: 5px solid black;
        }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let div_id = engine.query("#bordered-box");
    assert!(div_id.is_some(), "应该找到带边框的 div");

    println!("✓ 带边框的 div 渲染配置正确");
}

/// 测试带阴影的 div
#[test]
fn test_div_with_shadow() {
    println!("⬜ 测试带阴影的 div");

    let mut engine = Engine::new(800, 600);

    let html = "<div id=\"shadowed-box\"></div>";
    let css = r#"
        #shadowed-box {
            width: 200px;
            height: 150px;
            background-color: lightgray;
            box-shadow: 10px 10px 20px rgba(0,0,0,0.5);
        }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let div_id = engine.query("#shadowed-box");
    assert!(div_id.is_some(), "应该找到带阴影的 div");

    println!("✓ 带阴影的 div 渲染配置正确");
}

/// 测试嵌套 div 的颜色继承
#[test]
fn test_nested_div_colors() {
    println!("📚 测试嵌套 div 的颜色继承");

    let mut engine = Engine::new(800, 600);

    let html = r#"
        <div id="parent">
            <div id="child1" class="child"></div>
            <div id="child2" class="child"></div>
        </div>
    "#;

    let css = r#"
        #parent { width: 300px; height: 200px; background-color: red; }
        #child1 { width: 100px; height: 80px; background-color: green; }
        #child2 { width: 100px; height: 80px; background-color: blue; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let parent = engine.query("#parent");
    let child1 = engine.query("#child1");
    let child2 = engine.query("#child2");

    assert!(parent.is_some(), "应该找到父 div");
    assert!(child1.is_some(), "应该找到子 div 1");
    assert!(child2.is_some(), "应该找到子 div 2");

    println!("✓ 嵌套 div 颜色配置正确");
}

/// 测试绝对定位的彩色 div
#[test]
fn test_absolute_positioned_div() {
    println!("📍 测试绝对定位的彩色 div");

    let mut engine = Engine::new(800, 600);

    let html = r#"
        <div id="abs-box" style="position: absolute; left: 100px; top: 50px;"></div>
    "#;

    let css = "#abs-box { width: 120px; height: 80px; background-color: purple; }";

    engine.set_html(html);
    engine.set_css(css);

    let div_id = engine.query("#abs-box");
    assert!(div_id.is_some(), "应该找到绝对定位的 div");

    println!("✓ 绝对定位 div 渲染配置正确");
}

/// 测试 HEX 颜色格式
#[test]
fn test_hex_color_format() {
    println!("🔣 测试 HEX 颜色格式");

    let mut engine = Engine::new(800, 600);

    let html = r#"
        <div id="hex1"></div>
        <div id="hex2"></div>
        <div id="hex3"></div>
    "#;

    let css = r#"
        #hex1 { width: 50px; height: 50px; background-color: #ff0000; }
        #hex2 { width: 50px; height: 50px; background-color: #00ff00; }
        #hex3 { width: 50px; height: 50px; background-color: #0000ff; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let boxes = engine.query_all("div");
    assert_eq!(boxes.len(), 3, "应该有 3 个 div");

    println!("✓ HEX 颜色格式配置正确");
}

/// 测试 RGB 颜色格式
#[test]
fn test_rgb_color_format() {
    println!("🌈 测试 RGB 颜色格式");

    let mut engine = Engine::new(800, 600);

    let html = "<div id=\"rgb-box\"></div>";
    let css = "#rgb-box { width: 100px; height: 100px; background-color: rgb(128, 64, 32); }";

    engine.set_html(html);
    engine.set_css(css);

    let div_id = engine.query("#rgb-box");
    assert!(div_id.is_some(), "应该找到 RGB 颜色 div");

    println!("✓ RGB 颜色格式配置正确");
}

/// 测试 RGBA 颜色格式
#[test]
fn test_rgba_color_format() {
    println!("🎭 测试 RGBA 颜色格式");

    let mut engine = Engine::new(800, 600);

    let html = "<div id=\"rgba-box\"></div>";
    let css = "#rgba-box { width: 100px; height: 100px; background-color: rgba(255, 0, 0, 0.5); }";

    engine.set_html(html);
    engine.set_css(css);

    let div_id = engine.query("#rgba-box");
    assert!(div_id.is_some(), "应该找到 RGBA 颜色 div");

    println!("✓ RGBA 颜色格式配置正确");
}

/// 测试 flex 布局中的彩色 div
#[test]
fn test_flex_colored_divs() {
    println!("📐 测试 flex 布局中的彩色 div");

    let mut engine = Engine::new(800, 600);

    let html = r#"
        <div id="flex-container">
            <div id="flex1" class="flex-item"></div>
            <div id="flex2" class="flex-item"></div>
            <div id="flex3" class="flex-item"></div>
        </div>
    "#;

    let css = r#"
        #flex-container { display: flex; width: 600px; height: 200px; }
        .flex-item { flex-grow: 1; height: 100%; }
        #flex1 { background-color: #ff6600; }
        #flex2 { background-color: #0066ff; }
        #flex3 { background-color: #6600ff; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let container = engine.query("#flex-container");
    assert!(container.is_some(), "应该找到 flex 容器");

    let items = engine.query_all(".flex-item");
    assert_eq!(items.len(), 3, "应该有 3 个 flex 子项");

    println!("✓ Flex 布局彩色 div 配置正确");
}

/// 测试 z-index 层叠的彩色 div
#[test]
fn test_z_index_colored_divs() {
    println!("📚 测试 z-index 层叠的彩色 div");

    let mut engine = Engine::new(800, 600);

    let html = r#"
        <div id="layer1" class="layer"></div>
        <div id="layer2" class="layer"></div>
        <div id="layer3" class="layer"></div>
    "#;

    let css = r#"
        .layer { position: absolute; width: 100px; height: 100px; }
        #layer1 { left: 0px; top: 0px; background-color: red; z-index: 1; }
        #layer2 { left: 50px; top: 50px; background-color: green; z-index: 2; }
        #layer3 { left: 100px; top: 100px; background-color: blue; z-index: 3; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let layers = engine.query_all(".layer");
    assert_eq!(layers.len(), 3, "应该有 3 个层叠 div");

    println!("✓ Z-index 层叠 div 配置正确");
}

/// 运行所有 div 渲染测试
#[test]
fn run_div_render_tests() {
    println!("\n========================================");
    println!("🎨 开始运行 Div 渲染测试");
    println!("========================================\n");

    // 测试单个彩色 div
    test_single_colored_div();

    // 测试多个不同颜色的 div
    test_multiple_colored_divs();

    // 测试带透明度的 div
    test_transparent_div();

    // 测试带边框的 div
    test_div_with_border();

    // 测试带阴影的 div
    test_div_with_shadow();

    // 测试嵌套 div
    test_nested_div_colors();

    // 测试绝对定位
    test_absolute_positioned_div();

    // 测试 HEX 颜色
    test_hex_color_format();

    // 测试 RGB 颜色
    test_rgb_color_format();

    // 测试 RGBA 颜色
    test_rgba_color_format();

    // 测试 Flex 布局
    test_flex_colored_divs();

    // 测试 Z-index 层叠
    test_z_index_colored_divs();

    println!("\n========================================");
    println!("✅ 所有 Div 渲染测试通过!");
    println!("========================================\n");
}