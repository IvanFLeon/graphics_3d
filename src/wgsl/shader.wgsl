@group(0) @binding(0) var<uniform> camera : mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) mat_row_0: vec4<f32>,
    @location(2) mat_row_1: vec4<f32>,
    @location(3) mat_row_2: vec4<f32>,
    @location(4) mat_row_3: vec4<f32>,
    @location(5) color: vec4<f32>,
}

struct VertexOutput {
    @location(0) color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let transform = mat4x4<f32>(
        input.mat_row_0,
        input.mat_row_1,
        input.mat_row_2,
        input.mat_row_3,
    );

    output.color = input.color;
    output.position = camera * transform * input.position;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
