struct VertexInput {
    @location(0) position: vec2f,
    @location(1) uv: vec2f,
    @location(2) tex_idx: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
    @location(1) tex_idx: u32,
}

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let ndc_x = (input.position.x / 800.0) * 2.0 - 1.0;
    let ndc_y = -(input.position.y / 600.0) * 2.0 + 1.0;

    output.position = vec4f(ndc_x, ndc_y, 0.0, 1.0);
    output.uv = input.uv;
    output.tex_idx = input.tex_idx;

    return output;
}

@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var sampler: sampler;

struct FragmentInput {
    @location(0) uv: vec2f,
    @location(1) tex_idx: u32,
}

struct FragmentOutput {
    @location(0) color: vec4f,
}

@fragment
fn fragment_main(input: FragmentInput) -> FragmentOutput {
    var output: FragmentOutput;

    let sampled_color = textureSample(texture, sampler, input.uv);
    output.color = sampled_color;

    return output;
}
