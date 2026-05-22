// 布局项 - WGSL 侧表示 (与 Rust LayoutItem 对齐)
struct LayoutItem {
    // 基础属性
    size: vec2f,           // [width, height]
    margin: vec4f,          // [top, right, bottom, left]
    padding: vec4f,        // [top, right, bottom, left]
    pos: vec2f,             // [x, y]
    size_constraint: vec2f, // [right, bottom] - 绝对定位时用 right/bottom 约束尺寸

    // 布局类型
    flow_type: u32,         // 0=文档流, 1=绝对定位, 2=Flex, 3=Grid
    weight: f32,            // flex-grow
    flex_shrink: f32,       // flex-shrink
    z_index: f32,           // 层级

    // 纹理和状态
    tex_idx: u32,           // 纹理索引
    is_valid: u32,          // 是否启用 (0=禁用, 1=启用)
    is_hide: u32,           // 是否隐藏 (0=显示, 1=隐藏)

    // 样式
    bg_color: vec4f,        // [r, g, b, a]
    border: vec4f,          // [top, right, bottom, left] 边框宽度
    border_color: vec4f,    // [r, g, b, a]
    opacity: f32,           // 不透明度 0.0-1.0
    overflow: u32,          // 溢出处理: 0=visible, 1=hidden, 2=scroll, 3=auto

    // 变换
    transform: vec6f,       // 2D变换矩阵 [a, b, c, d, tx, ty]
                            // | a c tx |
                            // | b d ty |
                            // | 0 0 1  |

    // 阴影
    shadow_color: vec4f,    // [r, g, b, a]
    shadow_offset: vec2f,   // [x, y]
    shadow_blur: f32,       // 阴影模糊半径
    shadow_spread: f32,     // 阴影扩散
    has_shadow: u32,        // 是否有阴影 (0=无, 1=有)

    // 圆角
    border_radius: vec4f,   // [top-left, top-right, bottom-right, bottom-left]

    // Flexbox 属性
    visibility: u32,       // 可见性: 0=visible, 1=hidden, 2=collapse
    flex_direction: u32,    // flex方向: 0=row, 1=row-reverse, 2=column, 3=column-reverse
    flex_wrap: u32,         // flex换行: 0=nowrap, 1=wrap, 2=wrap-reverse
    align_items: u32,       // 交叉轴对齐: 0=stretch, 1=flex-start, 2=flex-end, 3=center, 4=baseline
    justify_content: u32,   // 主轴对齐: 0=flex-start, 1=flex-end, 2=center, 3=space-between, 4=space-around, 5=space-evenly

    // 额外属性
    align_content: u32,     // 多行对齐: 0=stretch, 1=flex-start, 2=flex-end, 3=center, 4=space-between, 5=space-around
    align_self: u32,        // 自身对齐: 0=auto, 1=flex-start, 2=flex-end, 3=center, 4=baseline, 5=stretch
    order: i32,             // 排序顺序
    flex_basis: f32,        // flex基础尺寸
    gap: f32,               // 间距

    // 交互/视觉效果
    outline_width: f32,
    outline_color: vec4f,
    cursor: u32,
    pointer_events: u32,

    // 浮动
    float: u32,
    clear: u32,
    overflow_x: u32,
    overflow_y: u32,

    // 文本/尺寸约束
    line_height: f32,
    letter_spacing: f32,
    word_spacing: f32,
    text_indent: f32,
    white_space: u32,
    max_width: f32,
    min_width: f32,
    max_height: f32,
    min_height: f32,
}

// 布局环境
struct LayoutEnv {
    view_size: vec2f,       // [width, height]
    flow_dir: u32,          // 0=垂直, 1=水平
    gap: vec2f,             // [row_gap, column_gap]
}

// 存储缓冲区绑定
@group(0) @binding(0) var<storage, read_write> items: array<LayoutItem>;
@group(0) @binding(1) var<uniform> env: LayoutEnv;

// 辅助函数：计算绝对定位元素的最终位置
fn compute_absolute_pos(item: &LayoutItem) -> vec2f {
    var final_pos = item.pos;

    // 如果有 size_constraint (right/bottom)，则重新计算位置
    if (item.size_constraint.x >= 0.0) {
        final_pos.x = env.view_size.x - item.size_constraint.x - item.size.x;
    }
    if (item.size_constraint.y >= 0.0) {
        final_pos.y = env.view_size.y - item.size_constraint.y - item.size.y;
    }

    return final_pos;
}

// 文档流布局 - 从上到下
fn layout_flow(index: u32, item: *mut LayoutItem, parent_pos: vec2f) {
    var current_y = parent_pos.y;

    // 遍历所有后续元素计算 Y 位置
    // 简化：每个元素之间间隔 margin.top + size.height + margin.bottom
    let item_ref = *item;

    let y_offset = item_ref.padding.x + item_ref.margin.x;
    (*item).pos = vec2f(parent_pos.x + item_ref.margin.w, current_y + y_offset);
}

// 绝对定位布局
fn layout_absolute(item: *mut LayoutItem) {
    (*item).pos = compute_absolute_pos(&(*item));
}

// Flex 布局计算
fn calculate_flex(layout: *mut LayoutItem, flex_items: array<LayoutItem>, item_count: u32) {
    let total_weight = 0.0;
    let available_space = (*layout).size.x;

    // 计算总权重
    var total = 0.0;
    for (var i = 0u; i < item_count; i++) {
        total += flex_items[i].weight;
    }

    // 按权重分配空间
    var x_offset = 0.0;
    for (var i = 0u; i < item_count; i++) {
        let allocated_width = (flex_items[i].weight / total) * available_space;
        (*layout).pos.x = x_offset;
        x_offset += allocated_width;
    }
}

// 布局计算主函数
@compute @workgroup_size(64)
fn layout_compute(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;

    // 检查索引是否有效
    if (index >= arrayLength(&items)) {
        return;
    }

    var item = &items[index];

    // 检查元素是否有效且可见
    if ((*item).is_valid == 0u || (*item).is_hide == 1u) {
        return;
    }

    // 根据不同的 flow_type 应用布局规则
    switch ((*item).flow_type) {
        case 0u: {  // 文档流
            layout_flow(index, item, vec2f(0.0, 0.0));
        }
        case 1u: {  // 绝对定位
            layout_absolute(item);
        }
        case 2u: {  // Flex - 简化实现
            // Flex 布局需要更复杂的实现，包括：
            // 1. 收集子元素
            // 2. 计算主轴和交叉轴
            // 3. 按权重分配空间
            // 此处简化处理
        }
        default: {
            // 保持原样
        }
    }
}