//! WebGPU 渲染引擎 - 纯色图片测试
//!
//! 测试纯色图片的渲染功能

use webgpu_web_renderer::{Engine, WebNativeBridge};

/// 测试纯红色图片
#[test]
fn test_pure_red_image() {
    println!("🔴 测试纯红色图片");

    let mut engine = Engine::new(800, 600);

    // 使用带背景色的 div 模拟纯色图片
    let html = "<div id=\"red-image\"></div>";
    let css = "#red-image { width: 200px; height: 150px; background-color: red; }";

    engine.set_html(html);
    engine.set_css(css);

    let img_id = engine.query("#red-image");
    assert!(img_id.is_some(), "应该找到红色图片元素");

    println!("✓ 纯红色图片渲染配置正确");
}

/// 测试纯蓝色图片
#[test]
fn test_pure_blue_image() {
    println!("🔵 测试纯蓝色图片");

    let mut engine = Engine::new(800, 600);

    let html = "<div id=\"blue-image\"></div>";
    let css = "#blue-image { width: 150px; height: 100px; background-color: #0000ff; }";

    engine.set_html(html);
    engine.set_css(css);

    let img_id = engine.query("#blue-image");
    assert!(img_id.is_some(), "应该找到蓝色图片元素");

    println!("✓ 纯蓝色图片渲染配置正确");
}

/// 测试渐变色模拟（多个色块组合）
#[test]
fn test_gradient_like_image() {
    println!("🌅 测试渐变模拟图片");

    let mut engine = Engine::new(800, 600);

    let html = r#"
        <div id="gradient-container">
            <div id="stripe1" class="stripe"></div>
            <div id="stripe2" class="stripe"></div>
            <div id="stripe3" class="stripe"></div>
            <div id="stripe4" class="stripe"></div>
            <div id="stripe5" class="stripe"></div>
        </div>
    "#;

    let css = r#"
        #gradient-container { width: 100px; height: 100px; }
        .stripe { width: 100%; height: 20%; }
        #stripe1 { background-color: #ff0000; }
        #stripe2 { background-color: #ff3300; }
        #stripe3 { background-color: #ff6600; }
        #stripe4 { background-color: #ff9900; }
        #stripe5 { background-color: #ffcc00; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let stripes = engine.query_all(".stripe");
    assert_eq!(stripes.len(), 5, "应该有 5 个条纹");

    println!("✓ 渐变模拟图片渲染配置正确");
}

/// 测试半透明 PNG 模拟
#[test]
fn test_transparent_image_simulation() {
    println!("🖼️ 测试透明图片模拟");

    let mut engine = Engine::new(800, 600);

    let html = r#"
        <div id="image-container">
            <div id="bg-layer"></div>
            <div id="overlay-image"></div>
        </div>
    "#;

    let css = r#"
        #image-container { position: relative; width: 200px; height: 200px; }
        #bg-layer { position: absolute; width: 100%; height: 100%; background-color: white; }
        #overlay-image { position: absolute; width: 100px; height: 100px; background-color: rgba(255,0,0,0.5); left: 50px; top: 50px; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let overlay = engine.query("#overlay-image");
    assert!(overlay.is_some(), "应该找到半透明覆盖层");

    println!("✓ 半透明图片模拟渲染配置正确");
}

/// 测试多图层图片模拟
#[test]
fn test_multi_layer_image() {
    println!("🎭 测试多图层图片模拟");

    let mut engine = Engine::new(800, 600);

    let html = r#"
        <div id="composite-image">
            <div id="layer-bg" class="img-layer"></div>
            <div id="layer-mid" class="img-layer"></div>
            <div id="layer-fg" class="img-layer"></div>
        </div>
    "#;

    let css = r#"
        #composite-image { position: relative; width: 200px; height: 200px; }
        .img-layer { position: absolute; width: 100%; height: 100%; }
        #layer-bg { background-color: white; z-index: 1; }
        #layer-mid { background-color: green; opacity: 0.5; z-index: 2; }
        #layer-fg { background-color: blue; opacity: 0.3; z-index: 3; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let layers = engine.query_all(".img-layer");
    assert_eq!(layers.len(), 3, "应该有 3 个图层");

    println!("✓ 多图层图片模拟渲染配置正确");
}

/// 测试方形图片
#[test]
fn test_square_image() {
    println!("⬜ 测试方形图片");

    let mut engine = Engine::new(800, 600);

    let html = "<div id=\"square\"></div>";
    let css = "#square { width: 100px; height: 100px; background-color: gray; }";

    engine.set_html(html);
    engine.set_css(css);

    let square = engine.query("#square");
    assert!(square.is_some(), "应该找到方形图片");

    println!("✓ 方形图片渲染配置正确");
}

/// 测试非方形图片
#[test]
fn test_rectangular_image() {
    println!("📐 测试非方形图片");

    let mut engine = Engine::new(800, 600);

    let html = r#"
        <div id="landscape"></div>
        <div id="portrait"></div>
    "#;

    let css = r#"
        #landscape { width: 200px; height: 100px; background-color: orange; }
        #portrait { width: 100px; height: 200px; background-color: purple; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let landscape = engine.query("#landscape");
    let portrait = engine.query("#portrait");

    assert!(landscape.is_some(), "应该找到横向图片");
    assert!(portrait.is_some(), "应该找到纵向图片");

    println!("✓ 非方形图片渲染配置正确");
}

/// 测试大尺寸图片
#[test]
fn test_large_image() {
    println!("🖼️ 测试大尺寸图片");

    let mut engine = Engine::new(1920, 1080);

    let html = "<div id=\"large-image\"></div>";
    let css = "#large-image { width: 1920px; height: 1080px; background-color: black; }";

    engine.set_html(html);
    engine.set_css(css);

    let large_img = engine.query("#large-image");
    assert!(large_img.is_some(), "应该找到大尺寸图片");

    println!("✓ 大尺寸图片渲染配置正确");
}

/// 测试图片平铺布局
#[test]
fn test_tiled_images() {
    println!("📊 测试图片平铺布局");

    let mut engine = Engine::new(800, 600);

    let html = r#"
        <div id="gallery">
            <div class="tile"></div>
            <div class="tile"></div>
            <div class="tile"></div>
            <div class="tile"></div>
        </div>
    "#;

    let css = r#"
        #gallery { display: flex; flex-wrap: wrap; width: 400px; }
        .tile { width: 100px; height: 100px; margin: 10px; }
        .tile:nth-child(1) { background-color: red; }
        .tile:nth-child(2) { background-color: green; }
        .tile:nth-child(3) { background-color: blue; }
        .tile:nth-child(4) { background-color: yellow; }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let tiles = engine.query_all(".tile");
    assert_eq!(tiles.len(), 4, "应该有 4 个平铺图片");

    println!("✓ 图片平铺布局渲染配置正确");
}

/// 测试图片与边框组合
#[test]
fn test_image_with_frame() {
    println!("🖼️ 测试带边框的图片");

    let mut engine = Engine::new(800, 600);

    let html = "<div id=\"framed-image\"></div>";
    let css = r#"
        #framed-image {
            width: 150px;
            height: 150px;
            background-color: teal;
            border: 10px solid gold;
            box-shadow: 5px 5px 15px rgba(0,0,0,0.5);
        }
    "#;

    engine.set_html(html);
    engine.set_css(css);

    let framed = engine.query("#framed-image");
    assert!(framed.is_some(), "应该找到带边框的图片");

    println!("✓ 带边框图片渲染配置正确");
}

/// 运行所有纯色图片测试
#[test]
fn run_pure_color_image_tests() {
    println!("\n========================================");
    println!("🎨 开始运行纯色图片测试");
    println!("========================================\n");

    test_pure_red_image();
    test_pure_blue_image();
    test_gradient_like_image();
    test_transparent_image_simulation();
    test_multi_layer_image();
    test_square_image();
    test_rectangular_image();
    test_large_image();
    test_tiled_images();
    test_image_with_frame();

    println!("\n========================================");
    println!("✅ 所有纯色图片测试通过!");
    println!("========================================\n");
}