struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
}

const GREEN: vec3<f32> = vec3<f32>(0.0, 1.0, 0.0);

// Fragment shader
@fragment
fn main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    return vec4<f32>(GREEN, 0.5);
}
