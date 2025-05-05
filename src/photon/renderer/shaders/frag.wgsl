struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
}

struct CameraUniform {
    view_position: vec4<f32>,
    view_proj: mat4x4<f32>,
};


@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(1) @binding(1)
var s_diffuse: sampler;

const WHITE: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);

// Fragment shader
@fragment
fn main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.uv_coords);

    // Ambient lighting
    let ambient_strength = 0.01;
    let ambient_color = WHITE * ambient_strength;

    // Diffuse lighting
    // FIXME: Fixed light needs to be changed to add multiple dynamic lights
    let light_dir = normalize(vec3<f32>(10.0, 10.0, 10.0) - in.world_position);
    let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse_color = WHITE * diffuse_strength;

    let result = (ambient_color + diffuse_color) * object_color.rgb;

    return vec4<f32>(result, object_color.a);
}
