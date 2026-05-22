// 顶点输入
struct VertexInput {
    @location(0) position: vec2f,  // 位置
    @location(1) uv: vec2f,        // 纹理坐标
    @location(2) tex_idx: u32,      // 纹理索引
    @location(3) color: vec4f,      // 颜色
}

// 顶点输出 / 片段输入
struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
    @location(1) tex_idx: u32,
    @location(2) color: vec4f,
    @location(3) opacity: f32,
    @location(4) border_radius: vec4f,
    @location(5) tex_offset: vec2f,
    @location(6) tex_size: vec2f,
}

// Uniform 数据
struct Uniforms {
    viewport_size: vec2f,
    time: f32,
    _pad: vec2f,
}

// 绑定组
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var texture_array: binding_array<texture_2d<f32>>;

// 顶点着色器
@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // 转换位置到 NDC 坐标
    let viewport = uniforms.viewport_size;
    let ndc_x = (input.position.x / viewport.x) * 2.0 - 1.0;
    let ndc_y = -(input.position.y / viewport.y) * 2.0 + 1.0;

    output.position = vec4f(ndc_x, ndc_y, 0.0, 1.0);
    output.uv = input.uv;
    output.tex_idx = input.tex_idx;
    output.color = input.color;
    output.opacity = input.color.a;  // 从颜色中提取透明度
    output.border_radius = vec4f(0.0); // 默认无圆角
    output.tex_offset = vec2f(0.0, 0.0);
    output.tex_size = vec2f(1.0, 1.0);

    return output;
}

// 片段着色器
@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4f {
    var color = input.color;

    // 如果有纹理索引 (>0)，采样纹理
    if (input.tex_idx > 0u) {
        let tex_idx = input.tex_idx - 1u;  // 纹理索引从 1 开始
        let sampled = textureSample(texture_array[tex_idx], texture_sampler, input.uv);
        
        // 混合纹理颜色和指定颜色
        color = mix(color, sampled, sampled.a);
    }

    // 应用透明度
    color.a *= input.opacity;

    // 简单的圆角裁剪 (需要传入顶点坐标配合)
    // TODO: 需要额外数据来实现真正的圆角裁剪

    return color;
}

// SDF 文字渲染 (基于距离场的抗锯齿文字)
fn sdf_text_render(uv: vec2f, texture: texture_2d<f32>, font_size: f32) -> f32 {
    let distance = textureSample(texture, texture_sampler, uv).r;
    
    // 抗锯齿计算
    let smoothing = 0.25 / font_size;
    return smoothstep(0.5 - smoothing, 0.5 + smoothing, distance);
}

// 阴影渲染
fn apply_shadow(color: vec4f, shadow_color: vec4f, shadow_offset: vec2f, shadow_blur: f32) -> vec4f {
    // 简化阴影：直接混合
    var result = color;
    
    if (shadow_blur > 0.0) {
        // 有阴影时，边缘变暗
        let shadow_strength = shadow_color.a * 0.5;
        result.rgb = mix(result.rgb, shadow_color.rgb, shadow_strength);
    }
    
    return result;
}

// 渐变背景
struct GradientUniform {
    start_color: vec4f,
    end_color: vec4f,
    direction: u32,  // 0=垂直, 1=水平, 2=对角
}

@group(0) @binding(3) var<uniform> gradient_uniform: GradientUniform;

fn calculate_gradient(uv: vec2f) -> vec4f {
    var t: f32 = 0.0;
    
    switch (gradient_uniform.direction) {
        case 0u: { t = uv.y; }           // 垂直
        case 1u: { t = uv.x; }           // 水平
        case 2u: { t = (uv.x + uv.y) * 0.5; }  // 对角
        default: { t = uv.y; }
    }
    
    return mix(gradient_uniform.start_color, gradient_uniform.end_color, t);
}