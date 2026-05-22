// 布局项 - WGSL 侧表示
struct LayoutItem {
    size: vec2f,        // [width, height]
    margin: vec4f,      // [top, right, bottom, left]
    padding: vec4f,     // [top, right, bottom, left]
    pos: vec2f,         // [x, y]
    flow_type: u32,     // 0=文档流, 1=绝对定位, 2=Flex, 3=Grid
    weight: f32,        // flex-grow, flex-shrink
    z_index: f32,       // 层级
    tex_idx: u32,       // 纹理索引
    is_valid: u32,      // 是否启用 (0=禁用, 1=启用)
    is_hide: u32,       // 是否隐藏 (0=显示, 1=隐藏)
}

// 布局环境
struct LayoutEnv {
    view_size: vec2f,   // [width, height]
    flow_dir: u32,      // 0=垂直, 1=水平
    gap: vec2f,         // [row_gap, column_gap]
}

// 存储缓冲区绑定
@group(0) @binding(0) var<storage, read_write> items: array<LayoutItem>;
@group(0) @binding(1) var<uniform> env: LayoutEnv;

// 辅助函数：获取子元素数量
fn get_child_count(node_index: u32) -> u32 {
    // 简化：假设 items 数组按树序排列
    // 实际实现需要更复杂的结构
    return 0u;
}

// 辅助函数：获取第一个子元素
fn get_first_child(node_index: u32) -> u32 {
    // 简化实现
    return node_index + 1u;
}

// 辅助函数：获取下一个兄弟元素
fn get_next_sibling(node_index: u32) -> u32 {
    // 简化实现
    return node_index + 1u;
}

// 文档流布局
fn layout_flow(item: ptr<function, LayoutItem>, parent_pos: vec2f) {
    // 文档流：从上到下，从左到右排布
    let item_ref = *item;

    // 计算实际尺寸
    let content_width = item_ref.size.x - item_ref.margin.y - item_ref.margin.w;
    let content_height = item_ref.size.y - item_ref.margin.x - item_ref.margin.z;

    // 更新位置（简化）
    (*item).pos = parent_pos + vec2f(item_ref.margin.w, item_ref.margin.x);
}

// 绝对定位布局
fn layout_absolute(item: ptr<function, LayoutItem>) {
    // 绝对定位：位置由样式直接指定
    // 这里不做修改，保持原有 pos
}

// Flex 布局
fn layout_flex(item: ptr<function, LayoutItem>, parent_size: vec2f) {
    // Flex 布局：按权重分配空间
    let item_ref = *item;

    // 简化的 flex 计算
    let available_space = parent_size.x;
    let allocated_width = item_ref.weight * available_space;

    // 更新尺寸
    (*item).size.x = allocated_width;
}

// 布局计算主函数
@compute @workgroup_size(64)
fn layout_compute(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;

    // 检查索引是否有效
    if (index >= arrayLength(&items)) {
        return;
    }

    let item = &items[index];

    // 检查元素是否有效且可见
    if (item.is_valid == 0u || item.is_hide == 1u) {
        return;
    }

    // 根据不同的 flow_type 应用布局规则
    switch (item.flow_type) {
        case 0u: {  // 文档流
            layout_flow(item, vec2f(0.0, 0.0));
        }
        case 1u: {  // 绝对定位
            layout_absolute(item);
        }
        case 2u: {  // Flex
            layout_flex(item, env.view_size);
        }
        default: {  // 其他
            // 保持原样
        }
    }
}