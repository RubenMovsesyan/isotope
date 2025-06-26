struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
}
struct CameraUniform {
    view_position: vec4<f32>,
    view_proj: mat4x4<f32>,
};

struct Light {
    position: vec3<f32>,
    normal: vec3<f32>,
    color: vec3<f32>,
    intensity: f32
};

struct LightsBuffer {
    data: array<Light>,
}

struct Material {
    ambient_color: vec3<f32>,
    diffuse_color: vec3<f32>,
    specular_color: vec3<f32>,
    specular_focus: f32,
    optical_density: f32,
    dissolve: f32,
    illum: u32,
    optional_texture: u32,
}

const BLUE: vec3<f32> = vec3<f32>(0.0, 0.0, 1.0);

// Fragment shader
@fragment
fn main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    return vec4<f32>(BLUE, 1.0);
}
