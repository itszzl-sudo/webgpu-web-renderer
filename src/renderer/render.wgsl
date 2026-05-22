struct VertexInput {
    @location(0) position: vec2f,
    @location(1) uv: vec2f,
    @location(2) tex_idx: u32,
    @location(3) color: vec4f,
}

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
}

struct Uniforms {
    viewport_size: vec2f,
    time: f32,
    _pad: vec2f,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let viewport = uniforms.viewport_size;
    let ndc_x = (input.position.x / viewport.x) * 2.0 - 1.0;
    let ndc_y = -(input.position.y / viewport.y) * 2.0 + 1.0;
    output.position = vec4f(ndc_x, ndc_y, 0.0, 1.0);
    output.color = input.color;
    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4f {
    return input.color;
}