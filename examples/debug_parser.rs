use webgpu_web_renderer::{Engine, WebNativeBridge};

fn main() {
    println!("调试 HTML 解析问题");
    
    let mut engine = Engine::new(800, 600);
    
    // 最简单的 HTML
    let simple_html = "<div>测试</div>";
    println!("输入 HTML: {}", simple_html);
    
    engine.set_html(simple_html);
    
    let all_nodes = engine.query_all("*");
    println!("解析结果: {} 个节点", all_nodes.len());
    
    let divs = engine.query_all("div");
    println!("DIV 元素: {} 个", divs.len());
    
    // 尝试查询
    let div_result = engine.query("div");
    println!("单个 DIV 查询: {:?}", div_result);
    
    // 检查 DOM 树状态
    println!("\n尝试不同的 HTML:");
    
    let test_cases = vec![
        "<p>段落</p>",
        "<span>文本</span>",
        "<h1>标题</h1>",
        "<div></div>",
    ];
    
    for (i, html) in test_cases.iter().enumerate() {
        let mut test_engine = Engine::new(800, 600);
        test_engine.set_html(html);
        let nodes = test_engine.query_all("*");
        println!("测试用例 {}: '{}' -> {} 个节点", i + 1, html, nodes.len());
    }
}