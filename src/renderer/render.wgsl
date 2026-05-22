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
    @location(2) tex_idx: u32,
}

// Uniform 数据
struct Uniforms {
    viewport_size: vec2f,
    time: f32,
    _pad: vec2f,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

// 纹理数组 + 采样器
@group(1) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(1) @binding(1) var sampler_linear: sampler;

// 顶点着色器
@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let viewport = uniforms.viewport_size;
    let ndc_x = (input.position.x / viewport.x) * 2.0 - 1.0;
    let ndc_y = -(input.position.y / viewport.y) * 2.0 + 1.0;

    output.position = vec4f(ndc_x, ndc_y, 0.0, 1.0);
    output.uv = input.uv;
    output.color = input.color;
    output.tex_idx = input.tex_idx;

    return output;
}

// 片段着色器
@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4f {
    var final_color = input.color;

    // 如果有纹理，进行采样混合
    if (input.tex_idx > 0u) {
        let tex_index = input.tex_idx - 1u;
        if (tex_index < arrayLength(&textures)) {
            let sampled = textureSample(textures[tex_index], sampler_linear, input.uv);
            // 混合：纹理颜色 * 顶点颜色
            final_color = sampled * input.color;
        }
    }

    return final_color;
}