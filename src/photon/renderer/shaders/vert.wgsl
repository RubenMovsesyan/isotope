struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct InstanceInput {
    @location(3) position: vec3<f32>,
    @location(4) rotation: vec4<f32>,
}

struct CameraUniform {
    view_position: vec4<f32>,
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// Vertex Shader
@vertex
fn main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv_coords = model.uv_coords;
    out.world_normal = model.normal;
    var world_position: vec4<f32> = vec4<f32>(model.position + instance.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;
    return out;
}
