// 布局项 - WGSL 侧表示 (必须与 Rust LayoutItem 内存布局完全一致)
struct LayoutItem {
    dom_id: u32,
    size: vec2f,
    margin: vec4f,
    padding: vec4f,
    pos: vec2f,
    size_constraint: vec2f,
    flow_type: u32,
    weight: f32,
    flex_shrink: f32,
    z_index: f32,
    tex_idx: u32,
    is_valid: u32,
    is_hide: u32,
    bg_color: vec4f,
    border: vec4f,
    border_color: vec4f,
    opacity: f32,
    overflow: u32,
    transform: array<f32, 6>,
    shadow_color: vec4f,
    shadow_offset: vec2f,
    shadow_blur: f32,
    shadow_spread: f32,
    has_shadow: u32,
    border_radius: vec4f,
    visibility: u32,
    flex_direction: u32,
    flex_wrap: u32,
    align_items: u32,
    justify_content: u32,
    align_content: u32,
    align_self: u32,
    order: i32,
    flex_basis: f32,
    gap: f32,
    outline_width: f32,
    outline_color: vec4f,
    cursor: u32,
    pointer_events: u32,
    float: u32,
    clear: u32,
    overflow_x: u32,
    overflow_y: u32,
    line_height: f32,
    letter_spacing: f32,
    word_spacing: f32,
    text_indent: f32,
    white_space: u32,
    max_width: f32,
    min_width: f32,
    max_height: f32,
    min_height: f32,
    clip_rect: vec4f,
    scroll_offset: vec2f,
}

// 布局环境
struct LayoutEnv {
    view_size: vec2f,
    flow_dir: u32,
    gap: vec2f,
}

// 存储缓冲区绑定
@group(0) @binding(0) var<storage, read_write> items: array<LayoutItem>;
@group(0) @binding(1) var<uniform> env: LayoutEnv;

// 计算数量（用于 workgroup 汇总）
struct AtomicResult {
    total_count: u32,
    total_weight: f32,
}
@group(0) @binding(2) var<storage, read_write> atomic_result: AtomicResult;

// 应用尺寸约束 (max/min width/height)
fn apply_size_constraints(item: ptr<function, LayoutItem>) {
    if ((*item).max_width > 0.0 && (*item).size.x > (*item).max_width) {
        (*item).size.x = (*item).max_width;
    }
    if ((*item).min_width > 0.0 && (*item).size.x < (*item).min_width) {
        (*item).size.x = (*item).min_width;
    }
    if ((*item).max_height > 0.0 && (*item).size.y > (*item).max_height) {
        (*item).size.y = (*item).max_height;
    }
    if ((*item).min_height > 0.0 && (*item).size.y < (*item).min_height) {
        (*item).size.y = (*item).min_height;
    }
}

// 计算绝对定位元素的最终位置
fn compute_absolute_pos(item: ptr<function, LayoutItem>) {
    if ((*item).size_constraint.x >= 0.0) {
        (*item).pos.x = env.view_size.x - (*item).size_constraint.x - (*item).size.x;
    }
    if ((*item).size_constraint.y >= 0.0) {
        (*item).pos.y = env.view_size.y - (*item).size_constraint.y - (*item).size.y;
    }
}

// 汇总阶段: 收集 workgroup 级别的统计数据
var<workgroup> wg_weight_sum: f32;
var<workgroup> wg_item_count: u32;

@compute @workgroup_size(64)
fn layout_compute(@builtin(global_invocation_id) global_id: vec3<u32>,
                  @builtin(local_invocation_id) local_id: vec3<u32>,
                  @builtin(workgroup_id) wg_id: vec3<u32>) {
    let index = global_id.x;
    let item_count = arrayLength(&items);

    // 第一阶段: 每个线程处理一个元素
    if (index < item_count) {
        var item = &items[index];

        if ((*item).is_valid == 0u || (*item).is_hide == 1u) {
            return;
        }

        // 1. 应用尺寸约束 (所有布局类型都适用)
        apply_size_constraints(item);

        // 2. 根据 flow_type 处理布局
        switch ((*item).flow_type) {
            case 0u: {  // 文档流 - 仅应用尺寸约束，位置由 CPU 计算
                // 文档流位置计算需要顺序处理，GPU 仅处理约束
            }
            case 1u: {  // 绝对定位 - 独立计算位置
                compute_absolute_pos(item);
            }
            case 2u: {  // Flex - 收集权重用于后续分配
                // 权重收集在第二阶段处理
            }
            default: {}
        }

        // 3. 收集统计信息到 workgroup 共享内存
        if ((*item).flow_type == 2u) {
            wg_weight_sum += (*item).weight;
            wg_item_count += 1u;
        }
    }

    // 同步 workgroup
    workgroupBarrier();

    // 第二个 workgroup 线程 (local_id.x == 0) 写入汇总结果
    if (local_id.x == 0u && wg_weight_sum > 0.0) {
        // atomic_result 累加所有 workgroup 的汇总
        // 简化: workgroup 0 写自己的汇总
        if (wg_id.x == 0u) {
            atomic_result.total_weight = wg_weight_sum;
            atomic_result.total_count = wg_item_count;
        }
    }
}