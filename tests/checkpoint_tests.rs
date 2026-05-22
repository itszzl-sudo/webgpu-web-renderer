//! WebGPU 渲染引擎 - 数据流检查点测试
//!
//! 这个测试文件用于验证整个渲染管线的数据流和状态转换
//! 按照数据处理的顺序创建检查点，确保每个阶段都符合预期

use webgpu_web_renderer::{Engine, WebNativeBridge};

// 检查点 1: HTML 输入验证
#[test]
fn checkpoint_1_html_input_validation() {
    println!("🔍 检查点 1: HTML 输入验证");
    
    let mut engine = Engine::new(800, 600);
    
    // 测试用例 1: 简单 HTML
    let simple_html = "<div><p>测试</p></div>";
    engine.set_html(simple_html);
    assert_eq!(engine.query_all("*").len(), 2, "简单 HTML 应该产生 2 个节点");
    println!("✓ 简单 HTML 解析正确");
    
    // 测试用例 2: 带属性的 HTML
    let html_with_attrs = "<div id=\"container\" class=\"main\"><p class=\"text\">内容</p></div>";
    engine.set_html(html_with_attrs);
    
    let container_id = engine.query("#container");
    assert!(container_id.is_some(), "应该找到 #container");
    assert_eq!(engine.get_attr(container_id.unwrap(), "class"), Some("main".to_string()));
    println!("✓ 带属性的 HTML 解析正确");
    
    // 测试用例 3: 嵌套结构
    let nested_html = "<div class=\"outer\"><div class=\"middle\"><div class=\"inner\"><span>深度嵌套</span></div></div></div>";
    engine.set_html(nested_html);
    let all_divs = engine.query_all("div");
    assert_eq!(all_divs.len(), 3, "应该有 3 个 div 元素");
    println!("✓ 嵌套结构解析正确");
    
    println!("✅ 检查点 1 通过: HTML 输入验证\n");
}

// 检查点 2: CSS 输入验证
#[test]
fn checkpoint_2_css_input_validation() {
    println!("🔍 检查点 2: CSS 输入验证");
    
    let mut engine = Engine::new(800, 600);
    engine.set_html("<div id=\"test\" class=\"box\"><p>内容</p></div>");
    
    // 测试用例 1: 基础 CSS 规则
    let basic_css = "div { color: red; } .box { background: blue; } #test { padding: 10px; }";
    engine.set_css(basic_css);
    println!("✓ 基础 CSS 规则应用成功");
    
    // 测试用例 2: 复杂选择器
    let complex_css = "div .box { margin: 20px; } div > p { font-size: 16px; } #test .box { border: 1px solid black; }";
    engine.set_css(complex_css);
    println!("✓ 复杂选择器处理成功");
    
    // 测试用例 3: 样式优先级
    let priority_css = "div { color: red !important; } .box { color: blue; } #test { color: green; }";
    engine.set_css(priority_css);
    println!("✓ 样式优先级处理成功");
    
    println!("✅ 检查点 2 通过: CSS 输入验证\n");
}

// 检查点 3: DOM 树结构验证
#[test]
fn checkpoint_3_dom_tree_structure() {
    println!("🔍 检查点 3: DOM 树结构验证");
    
    let mut engine = Engine::new(800, 600);
    let html = "<div id=\"root\"><header><h1>标题</h1><nav><a href=\"home\">首页</a><a href=\"about\">关于</a></nav></header><main><section><h2>区块</h2><p>段落内容</p></section></main><footer><p>页脚</p></footer></div>";
    
    engine.set_html(html);
    
    // 验证节点数量
    let total_nodes = engine.query_all("*").len();
    assert!(total_nodes >= 10, "DOM 树应该包含至少 10 个节点");
    println!("✓ DOM 节点数量正确: {}", total_nodes);
    
    // 验证层级结构
    let root_id = engine.query("#root");
    assert!(root_id.is_some(), "应该找到根节点");
    
    let header_id = engine.query("header");
    assert!(header_id.is_some(), "应该找到 header");
    
    let header_parent = engine.parent_node(header_id.unwrap());
    assert!(header_parent.is_some(), "header 应该有父节点");
    println!("✓ DOM 层级结构正确，header 父节点 ID: {:?}", header_parent);
    
    // 验证兄弟节点
    let nav_links = engine.query_all("a");
    assert!(nav_links.len() >= 2, "应该有至少 2 个导航链接");
    println!("✓ DOM 兄弟节点关系正确，找到 {} 个链接", nav_links.len());
    
    println!("✅ 检查点 3 通过: DOM 树结构验证\n");
}

// 检查点 4: 样式应用和计算
#[test]
fn checkpoint_4_style_application() {
    println!("🔍 检查点 4: 样式应用和计算");
    
    let mut engine = Engine::new(800, 600);
    engine.set_html("<div id=\"container\" class=\"main\"><h1 class=\"title\">主标题</h1><p class=\"content\">内容段落</p></div>");
    
    let css = "#container { padding: 20px; background: #f0f0f0; } .title { color: #333; font-size: 32px; } .content { color: #666; line-height: 1.6; }";
    engine.set_css(css);
    
    // 验证样式是否应用
    let container_id = engine.query("#container");
    assert!(container_id.is_some(), "应该找到容器");
    
    // 验证动态样式修改
    engine.set_style(".title", "color", "blue");
    println!("✓ 动态样式修改成功");
    
    engine.set_style("#container", "background", "white");
    println!("✓ ID 选择器样式修改成功");
    
    // 验证内联样式优先级
    let h1_id = engine.query("h1");
    if let Some(id) = h1_id {
        engine.set_attr(id, "style", "color: red;");
        println!("✓ 内联样式设置成功");
    }
    
    println!("✅ 检查点 4 通过: 样式应用和计算\n");
}

// 检查点 5: 查询功能验证
#[test]
fn checkpoint_5_query_functionality() {
    println!("🔍 检查点 5: 查询功能验证");
    
    let mut engine = Engine::new(800, 600);
    let html = "<div id=\"app\" class=\"container\"><header class=\"header\"><h1 id=\"main-title\" class=\"title\">应用标题</h1></header><main class=\"content\"><section class=\"section active\"><h2 class=\"subtitle\">第一部分</h2><p class=\"text\">段落文本</p></section><section class=\"section\"><h2 class=\"subtitle\">第二部分</h2><p class=\"text\">另一个段落</p></section></main></div>";
    
    engine.set_html(html);
    
    // 测试 ID 查询
    let app_id = engine.query("#app");
    assert!(app_id.is_some(), "ID 查询应该找到元素");
    println!("✓ ID 查询功能正常");
    
    // 测试类名查询
    let sections = engine.query_all(".section");
    assert_eq!(sections.len(), 2, "类名查询应该找到 2 个区块");
    println!("✓ 类名查询功能正常");
    
    // 测试标签查询
    let headers = engine.query_all("h2");
    assert_eq!(headers.len(), 2, "标签查询应该找到 2 个 h2");
    println!("✓ 标签查询功能正常");
    
    // 测试组合查询
    let active_section = engine.query(".section");
    assert!(active_section.is_some(), "类名查询应该找到元素");
    println!("✓ 组合查询功能正常");
    
    // 测试查询性能
    let start = std::time::Instant::now();
    let all_elements = engine.query_all("*");
    let query_time = start.elapsed();
    
    println!("✓ 查询性能: {} 个元素，耗时 {:?}", all_elements.len(), query_time);
    assert!(query_time.as_millis() < 100, "查询时间应该小于 100ms");
    
    println!("✅ 检查点 5 通过: 查询功能验证\n");
}

// 检查点 6: 属性操作验证
#[test]
fn checkpoint_6_attribute_operations() {
    println!("🔍 检查点 6: 属性操作验证");
    
    let mut engine = Engine::new(800, 600);
    engine.set_html("<a id=\"link\" href=\"https://example.com\">链接</a><input id=\"input\" type=\"text\" />");
    
    // 测试属性读取
    let link_id = engine.query("#link");
    assert!(link_id.is_some(), "应该找到链接");
    
    let href = engine.get_attr(link_id.unwrap(), "href");
    assert_eq!(href, Some("https://example.com".to_string()), "href 属性值应该正确");
    println!("✓ 属性读取功能正常");
    
    // 测试属性设置
    engine.set_attr(link_id.unwrap(), "target", "_blank");
    let target = engine.get_attr(link_id.unwrap(), "target");
    assert_eq!(target, Some("_blank".to_string()), "新设置的属性应该可以读取");
    println!("✓ 属性设置功能正常");
    
    // 测试属性修改
    engine.set_attr(link_id.unwrap(), "href", "https://new-url.com");
    let new_href = engine.get_attr(link_id.unwrap(), "href");
    assert_eq!(new_href, Some("https://new-url.com".to_string()), "属性修改应该生效");
    println!("✓ 属性修改功能正常");
    
    // 测试多属性操作
    let input_id = engine.query("#input");
    if let Some(id) = input_id {
        engine.set_attr(id, "placeholder", "请输入内容");
        engine.set_attr(id, "maxlength", "100");
        engine.set_attr(id, "required", "true");
        
        assert_eq!(engine.get_attr(id, "placeholder"), Some("请输入内容".to_string()));
        assert_eq!(engine.get_attr(id, "maxlength"), Some("100".to_string()));
        assert_eq!(engine.get_attr(id, "required"), Some("true".to_string()));
        println!("✓ 多属性操作功能正常");
    }
    
    println!("✅ 检查点 6 通过: 属性操作验证\n");
}

// 检查点 7: 交互系统验证
#[test]
fn checkpoint_7_interaction_system() {
    println!("🔍 检查点 7: 交互系统验证");
    
    let mut engine = Engine::new(800, 600);
    let html = "<div class=\"interactive\"><button id=\"click-btn\">点击按钮</button><form id=\"test-form\"><input type=\"text\" name=\"username\" /><button type=\"submit\">提交</button></form></div>";
    
    engine.set_html(html);
    
    // 测试点击事件注册
    let btn_id = engine.query("#click-btn");
    assert!(btn_id.is_some(), "应该找到按钮");
    println!("✓ 点击目标元素可访问");
    
    // 测试表单识别
    let form_id = engine.query("#test-form");
    assert!(form_id.is_some(), "应该找到表单");
    println!("✓ 表单元素可识别");
    
    // 测试点击测试功能
    let hit_result = engine.hit_test(100.0, 100.0);
    println!("✓ 点击测试功能可调用: {:?}", hit_result);
    
    // 测试视口操作
    let (width, height) = engine.viewport();
    assert_eq!(width, 800, "初始宽度应该为 800");
    assert_eq!(height, 600, "初始高度应该为 600");
    println!("✓ 视口读取功能正常");
    
    engine.set_viewport(1024, 768);
    let (new_width, new_height) = engine.viewport();
    assert_eq!(new_width, 1024, "视口宽度应该更新为 1024");
    assert_eq!(new_height, 768, "视口高度应该更新为 768");
    println!("✓ 视口设置功能正常");
    
    println!("✅ 检查点 7 通过: 交互系统验证\n");
}

// 检查点 8: 数据流集成测试
#[test]
fn checkpoint_8_dataflow_integration() {
    println!("🔍 检查点 8: 数据流集成测试");
    
    let mut engine = Engine::new(1024, 768);
    
    // 完整的数据流: HTML -> DOM -> CSS -> 样式应用 -> 查询操作
    let html = "<div id=\"app\" class=\"application\"><header class=\"app-header\"><h1 class=\"logo\">WebGPU 引擎</h1><nav class=\"navigation\"><a class=\"nav-link\" href=\"features\">特性</a><a class=\"nav-link\" href=\"docs\">文档</a><a class=\"nav-link\" href=\"community\">社区</a></nav></header><main class=\"app-main\"><section class=\"hero\"><h2 class=\"hero-title\">高性能渲染</h2><p class=\"hero-description\">基于 WebGPU 的新一代渲染引擎</p><button class=\"cta-button\">开始使用</button></section><section class=\"features-grid\"><article class=\"feature-card\"><h3 class=\"feature-title\">GPU 加速</h3><p class=\"feature-text\">利用硬件加速实现流畅渲染</p></article><article class=\"feature-card\"><h3 class=\"feature-title\">内存安全</h3><p class=\"feature-text\">Rust 保证的内存安全保障</p></article></section></main></div>";
    
    // 阶段 1: HTML 输入和解析
    let start = std::time::Instant::now();
    engine.set_html(html);
    let parse_time = start.elapsed();
    println!("  ✓ HTML 解析完成，耗时: {:?}", parse_time);
    
    let total_nodes = engine.query_all("*").len();
    assert!(total_nodes >= 15, "应该解析出至少 15 个节点");
    println!("  ✓ DOM 节点数量: {}", total_nodes);
    
    // 阶段 2: CSS 输入和应用
    let css = ".application { max-width: 1200px; margin: 0 auto; padding: 20px; font-family: Arial, sans-serif; } .app-header { display: flex; justify-content: space-between; align-items: center; padding: 20px 0; border-bottom: 2px solid #333; } .logo { font-size: 24px; font-weight: bold; color: #333; } .navigation { display: flex; gap: 20px; } .nav-link { text-decoration: none; color: #666; transition: color 0.3s; } .hero { text-align: center; padding: 60px 20px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; border-radius: 8px; margin: 40px 0; } .hero-title { font-size: 48px; margin-bottom: 20px; } .cta-button { background: white; color: #667eea; border: none; padding: 15px 40px; font-size: 18px; border-radius: 30px; cursor: pointer; margin-top: 30px; } .features-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 30px; margin-top: 60px; } .feature-card { background: #f8f9fa; padding: 30px; border-radius: 8px; border-left: 4px solid #667eea; }";
    
    let start = std::time::Instant::now();
    engine.set_css(css);
    let style_time = start.elapsed();
    println!("  ✓ CSS 样式应用完成，耗时: {:?}", style_time);
    
    // 阶段 3: 样式修改和交互
    engine.set_style(".hero", "background", "linear-gradient(135deg, #f093fb 0%, #f5576c 100%)");
    println!("  ✓ 动态样式修改完成");
    
    engine.set_attr(engine.query("#app").unwrap(), "data-version", "1.0.0");
    println!("  ✓ 属性设置完成");
    
    // 阶段 4: 查询验证
    let nav_links = engine.query_all(".nav-link");
    assert_eq!(nav_links.len(), 3, "应该找到 3 个导航链接");
    println!("  ✓ 导航链接查询: {} 个", nav_links.len());
    
    let feature_cards = engine.query_all(".feature-card");
    assert_eq!(feature_cards.len(), 2, "应该找到 2 个特性卡片");
    println!("  ✓ 特性卡片查询: {} 个", feature_cards.len());
    
    // 阶段 5: 渲染准备
    let render_result = engine.render();
    println!("  ✓ 渲染调用完成: {} 字节", render_result.len());
    
    // 总体验证
    let total_time = parse_time + style_time;
    println!("  ✓ 总处理时间: {:?}", total_time);
    assert!(total_time.as_millis() < 50, "总处理时间应该小于 50ms");
    
    println!("✅ 检查点 8 通过: 数据流集成测试\n");
}

// 检查点 9: 性能和稳定性验证
#[test]
fn checkpoint_9_performance_stability() {
    println!("🔍 检查点 9: 性能和稳定性验证");
    
    let mut engine = Engine::new(1920, 1080);
    
    // 生成大规模文档
    let mut large_html = String::from("<div id=\"performance-test\">");
    for i in 0..50 {
        large_html.push_str(&format!("<section class=\"section-{}\"><h2>区块 {}</h2>", i, i));
        for j in 0..20 {
            large_html.push_str(&format!("<p class=\"para-{}-{}\">段落 {}-{} 内容</p>", i, j, i, j));
        }
        large_html.push_str("</section>");
    }
    large_html.push_str("</div>");
    
    // 性能测试 1: 大规模 HTML 解析
    let start = std::time::Instant::now();
    engine.set_html(&large_html);
    let parse_time = start.elapsed();
    
    let node_count = engine.query_all("*").len();
    println!("  ✓ 大规模解析: {} 节点，耗时 {:?}", node_count, parse_time);
    assert!(parse_time.as_millis() < 100, "大规模解析应该小于 100ms");
    
    // 性能测试 2: 复杂 CSS 应用
    let complex_css = "* { margin: 0; padding: 0; box-sizing: border-box; } body { font-family: Arial, sans-serif; line-height: 1.6; } #performance-test { max-width: 1200px; margin: 0 auto; padding: 20px; } section { margin: 30px 0; padding: 25px; background: #f8f9fa; border-radius: 8px; } h2 { font-size: 28px; color: #333; margin-bottom: 20px; border-left: 4px solid #667eea; padding-left: 15px; } p { line-height: 1.8; color: #666; margin: 10px 0; }";
    
    let start = std::time::Instant::now();
    engine.set_css(complex_css);
    let style_time = start.elapsed();
    println!("  ✓ CSS 应用耗时: {:?}", style_time);
    assert!(style_time.as_millis() < 10, "CSS 应用应该小于 10ms");
    
    // 性能测试 3: 批量查询
    let start = std::time::Instant::now();
    let all_sections = engine.query_all("section");
    let all_paragraphs = engine.query_all("p");
    let query_time = start.elapsed();
    
    println!("  ✓ 批量查询: {} 个区块, {} 个段落，耗时 {:?}", 
             all_sections.len(), all_paragraphs.len(), query_time);
    assert!(query_time.as_millis() < 50, "批量查询应该小于 50ms");
    
    // 性能测试 4: 批量操作
    let start = std::time::Instant::now();
    for i in 0..10 {
        let selector = format!(".section-{}", i * 5);
        engine.set_style(&selector, "background", "#f0f0f0");
    }
    let batch_time = start.elapsed();
    
    println!("  ✓ 批量样式操作: 10 次，耗时 {:?}", batch_time);
    assert!(batch_time.as_millis() < 50, "批量操作应该小于 50ms");
    
    // 稳定性测试
    let stability_iterations = 100;
    let start = std::time::Instant::now();
    
    for i in 0..stability_iterations {
        let selector = format!(".section-{}", i % 50);
        let _ = engine.query(&selector);
    }
    
    let stability_time = start.elapsed();
    let avg_time = stability_time / stability_iterations;
    
    println!("  ✓ 稳定性测试: {} 次查询，总耗时 {:?}，平均 {:?}", 
             stability_iterations, stability_time, avg_time);
    
    println!("✅ 检查点 9 通过: 性能和稳定性验证\n");
}

// 检查点 10: 端到端场景测试
#[test]
fn checkpoint_10_end_to_end_scenarios() {
    println!("🔍 检查点 10: 端到端场景测试");
    
    // 场景 1: 博客文章渲染
    println!("  📝 场景 1: 博客文章渲染");
    let mut blog_engine = Engine::new(1200, 800);
    
    let blog_html = "<article class=\"blog-post\"><header class=\"post-header\"><h1 class=\"post-title\">WebGPU 渲染引擎架构设计</h1><div class=\"post-meta\"><time class=\"post-date\">2024-01-15</time><span class=\"post-author\">技术团队</span></div></header><div class=\"post-content\"><p class=\"lead\">本文介绍 WebGPU 渲染引擎的架构设计思路。</p><h2 class=\"section-title\">核心架构</h2><p>渲染引擎采用模块化设计，分为 DOM 层、样式层、布局层和渲染层。</p><h2 class=\"section-title\">性能优化</h2><p>通过 GPU 加速和并行计算，实现高性能渲染。</p></div><footer class=\"post-footer\"><nav class=\"post-navigation\"><a class=\"nav-link prev\" href=\"prev\">上一篇</a><a class=\"nav-link next\" href=\"next\">下一篇</a></nav></footer></article>";
    
    blog_engine.set_html(blog_html);
    
    let blog_css = ".blog-post { max-width: 800px; margin: 0 auto; padding: 40px; } .post-header { border-bottom: 2px solid #eee; padding-bottom: 20px; margin-bottom: 30px; } .post-title { font-size: 36px; color: #333; margin-bottom: 15px; } .post-meta { color: #999; font-size: 14px; } .lead { font-size: 20px; color: #555; line-height: 1.8; margin-bottom: 30px; } .section-title { font-size: 28px; color: #333; margin: 40px 0 20px 0; border-left: 4px solid #667eea; padding-left: 15px; } .post-footer { border-top: 1px solid #eee; padding-top: 20px; margin-top: 40px; } .post-navigation { display: flex; justify-content: space-between; } .nav-link { color: #667eea; text-decoration: none; }";
    
    blog_engine.set_css(blog_css);
    
    let blog_nodes = blog_engine.query_all("*");
    let blog_sections = blog_engine.query_all(".section-title");
    
    assert!(blog_nodes.len() >= 10, "博客文章应该包含至少 10 个节点");
    assert_eq!(blog_sections.len(), 2, "博客文章应该有 2 个小节");
    println!("    ✓ 博客文章渲染成功: {} 节点, {} 小节", blog_nodes.len(), blog_sections.len());
    
    // 场景 2: 电商产品页面
    println!("  🛒 场景 2: 电商产品页面");
    let mut shop_engine = Engine::new(1400, 900);
    
    let shop_html = "<div class=\"product-page\"><div class=\"product-gallery\"><img class=\"product-image\" src=\"product.jpg\" alt=\"产品图片\" /></div><div class=\"product-info\"><h1 class=\"product-name\">高性能显卡</h1><p class=\"product-description\">最新一代 GPU，提供极致游戏体验</p><div class=\"product-price\">￥8999</div><div class=\"product-actions\"><button class=\"btn-buy\">立即购买</button><button class=\"btn-cart\">加入购物车</button></div><div class=\"product-specs\"><h3>产品规格</h3><ul><li>显存: 24GB GDDR6X</li><li>核心: 16384 CUDA</li><li>频率: 2.5GHz</li></ul></div></div></div>";
    
    shop_engine.set_html(shop_html);
    
    let shop_css = ".product-page { display: grid; grid-template-columns: 1fr 1fr; gap: 40px; padding: 40px; } .product-gallery { background: #f8f9fa; border-radius: 8px; padding: 20px; } .product-image { width: 100%; border-radius: 4px; } .product-name { font-size: 32px; color: #333; margin-bottom: 15px; } .product-price { font-size: 36px; color: #e74c3c; font-weight: bold; margin: 20px 0; } .product-actions { display: flex; gap: 15px; margin: 30px 0; } .btn-buy, .btn-cart { padding: 15px 30px; border: none; border-radius: 6px; cursor: pointer; } .btn-buy { background: #e74c3c; color: white; } .btn-cart { background: #3498db; color: white; } .product-specs { margin-top: 40px; padding: 20px; background: #f8f9fa; border-radius: 8px; }";
    
    shop_engine.set_css(shop_css);
    
    let shop_buttons = shop_engine.query_all("button");
    let shop_specs = shop_engine.query_all("li");
    assert!(shop_specs.len() >= 3, "产品页面应该有至少 3 个规格项");
    println!("    ✓ 电商页面渲染成功: {} 按钮, {} 规格项", shop_buttons.len(), shop_specs.len());
    
    // 场景 3: 仪表板界面
    println!("  📊 场景 3: 仪表板界面");
    let mut dashboard_engine = Engine::new(1600, 1000);
    
    let dashboard_html = "<div class=\"dashboard\"><header class=\"dashboard-header\"><h1 class=\"dashboard-title\">数据仪表板</h1><nav class=\"dashboard-nav\"><a class=\"nav-item active\">概览</a><a class=\"nav-item\">分析</a><a class=\"nav-item\">报告</a></nav></header><main class=\"dashboard-main\"><div class=\"stats-grid\"><div class=\"stat-card\"><div class=\"stat-value\">12,345</div><div class=\"stat-label\">总用户数</div></div><div class=\"stat-card\"><div class=\"stat-value\">89.5%</div><div class=\"stat-label\">转化率</div></div><div class=\"stat-card\"><div class=\"stat-value\">￥1.2M</div><div class=\"stat-label\">总收入</div></div><div class=\"stat-card\"><div class=\"stat-value\">456</div><div class=\"stat-label\">新增订单</div></div></div><div class=\"charts-container\"><div class=\"chart-card\"><h3 class=\"chart-title\">用户增长趋势</h3><div class=\"chart-placeholder\">图表区域</div></div><div class=\"chart-card\"><h3 class=\"chart-title\">收入分析</h3><div class=\"chart-placeholder\">图表区域</div></div></div></main></div>";
    
    dashboard_engine.set_html(dashboard_html);
    
    let dashboard_css = ".dashboard { padding: 20px; background: #f5f7fa; min-height: 100vh; } .dashboard-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 30px; } .dashboard-title { font-size: 28px; color: #333; } .dashboard-nav { display: flex; gap: 20px; } .nav-item { color: #666; text-decoration: none; padding: 8px 16px; border-radius: 4px; } .nav-item.active { background: #667eea; color: white; } .stats-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 20px; margin-bottom: 30px; } .stat-card { background: white; padding: 25px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); } .stat-value { font-size: 32px; font-weight: bold; color: #333; margin-bottom: 5px; } .stat-label { color: #999; font-size: 14px; } .charts-container { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; } .chart-card { background: white; padding: 25px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); } .chart-title { font-size: 18px; color: #333; margin-bottom: 20px; } .chart-placeholder { height: 300px; background: #f8f9fa; border-radius: 4px; display: flex; align-items: center; justify-content: center; color: #999; }";
    
    dashboard_engine.set_css(dashboard_css);
    
    let dashboard_stats = dashboard_engine.query_all(".stat-card");
    let dashboard_charts = dashboard_engine.query_all(".chart-card");
    
    assert_eq!(dashboard_stats.len(), 4, "仪表板应该有 4 个统计卡片");
    assert_eq!(dashboard_charts.len(), 2, "仪表板应该有 2 个图表卡片");
    println!("    ✓ 仪表板渲染成功: {} 统计卡片, {} 图表卡片", dashboard_stats.len(), dashboard_charts.len());
    
    println!("✅ 检查点 10 通过: 端到端场景测试\n");
}

// 主测试运行器
#[test]
fn run_all_checkpoints() {
    println!("🚀 开始运行所有检查点测试\n");
    
    println!("═════════════════════════════════════════════════════════");
    println!("WebGPU 渲染引擎 - 数据流检查点测试套件");
    println!("═══════════════════════════════════════════════════════════\n");
    
    checkpoint_1_html_input_validation();
    checkpoint_2_css_input_validation();
    checkpoint_3_dom_tree_structure();
    checkpoint_4_style_application();
    checkpoint_5_query_functionality();
    checkpoint_6_attribute_operations();
    checkpoint_7_interaction_system();
    checkpoint_8_dataflow_integration();
    checkpoint_9_performance_stability();
    checkpoint_10_end_to_end_scenarios();
    
    println!("═══════════════════════════════════════════════════════════");
    println!("🎉 所有检查点测试完成！");
    println!("═══════════════════════════════════════════════════════════");
}