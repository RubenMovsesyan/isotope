struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
}
struct CameraUniform {
    view_position: vec4<f32>,
    view_proj: mat4x4<f32>,
};

override RED: f32 = 1.0;
override GREEN: f32 = 1.0;
override BLUE: f32 = 1.0;

// Fragment shader
@fragment
fn main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    return vec4<f32>(RED, GREEN, BLUE, 1.0);
}
