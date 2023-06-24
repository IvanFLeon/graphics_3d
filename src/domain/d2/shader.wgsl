@group(0) @binding(0) var<uniform> transform : mat4x4<f32>;

struct Inputs {
    @location(0) position: vec4<f32>,
    @location(1) mat_row_0: vec4<f32>,
    @location(2) mat_row_1: vec4<f32>,
    @location(3) mat_row_2: vec4<f32>,
    @location(4) mat_row_3: vec4<f32>,
}

@vertex
fn vs_main(
    inputs: Inputs
) -> @builtin(position) vec4<f32> {

    let local_transform = mat4x4<f32>(
        inputs.mat_row_0,
        inputs.mat_row_1,
        inputs.mat_row_2,
        inputs.mat_row_3,
    );

    var pos : vec4<f32> = transform * local_transform * inputs.position;
    return pos;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1., 0., 0., 0.);
}
