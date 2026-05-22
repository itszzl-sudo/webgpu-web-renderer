// 顶点输入
struct VertexInput {
    @location(0) position: vec2f,
    @location(1) uv: vec2f,
    @location(2) tex_idx: u32,
    @location(3) color: vec4f,
}

// 顶点输出 / 片段输入
struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
    @location(1) color: vec4f,
}

// Uniform 数据
struct Uniforms {
    viewport_size: vec2f,
    time: f32,
    _pad: vec2f,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

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
    output.color = input.color;

    return output;
}

// 片段着色器
@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4f {
    return input.color;
}