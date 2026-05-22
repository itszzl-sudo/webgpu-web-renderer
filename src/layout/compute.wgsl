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

struct LayoutEnv {
    view_size: vec2f,
    flow_dir: u32,
    gap: vec2f,
}

@group(0) @binding(0) var<storage, read_write> items: array<LayoutItem>;
@group(0) @binding(1) var<uniform> env: LayoutEnv;

struct AtomicResult {
    total_count: u32,
    total_weight: f32,
}
@group(0) @binding(2) var<storage, read_write> atomic_result: AtomicResult;

// 应用尺寸约束
fn apply_size_constraints(index: u32) {
    if (items[index].max_width > 0.0 && items[index].size.x > items[index].max_width) {
        items[index].size.x = items[index].max_width;
    }
    if (items[index].min_width > 0.0 && items[index].size.x < items[index].min_width) {
        items[index].size.x = items[index].min_width;
    }
    if (items[index].max_height > 0.0 && items[index].size.y > items[index].max_height) {
        items[index].size.y = items[index].max_height;
    }
    if (items[index].min_height > 0.0 && items[index].size.y < items[index].min_height) {
        items[index].size.y = items[index].min_height;
    }
}

// 计算绝对定位
fn compute_absolute_pos(index: u32) {
    if (items[index].size_constraint.x >= 0.0) {
        items[index].pos.x = env.view_size.x - items[index].size_constraint.x - items[index].size.x;
    }
    if (items[index].size_constraint.y >= 0.0) {
        items[index].pos.y = env.view_size.y - items[index].size_constraint.y - items[index].size.y;
    }
}

var<workgroup> wg_weight_sum: f32;
var<workgroup> wg_item_count: u32;

@compute @workgroup_size(64)
fn layout_compute(@builtin(global_invocation_id) global_id: vec3<u32>,
                  @builtin(local_invocation_id) local_id: vec3<u32>,
                  @builtin(workgroup_id) wg_id: vec3<u32>) {
    let index = global_id.x;
    let item_count = arrayLength(&items);

    if (index < item_count) {
        if (items[index].is_valid == 0u || items[index].is_hide == 1u) {
            return;
        }

        // 1. 应用尺寸约束
        apply_size_constraints(index);

        // 2. 根据 flow_type 处理布局
        switch (items[index].flow_type) {
            case 0u: {}
            case 1u: {
                compute_absolute_pos(index);
            }
            case 2u: {}
            default: {}
        }

        // 3. 收集权重
        if (items[index].flow_type == 2u) {
            wg_weight_sum += items[index].weight;
            wg_item_count += 1u;
        }
    }

    workgroupBarrier();

    if (local_id.x == 0u && wg_weight_sum > 0.0) {
        if (wg_id.x == 0u) {
            atomic_result.total_weight = wg_weight_sum;
            atomic_result.total_count = wg_item_count;
        }
    }
}