struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

struct CameraUniform {
    view_position: vec4<f32>,
    view_proj: mat4x4<f32>,
}

struct Light {
    position: vec3<f32>,
    normal: vec3<f32>,
    color: vec3<f32>,
    intensity: f32,
}

const CAMERA_BIND_GROUP: u32 = 0;
const LIGHT_BIND_GROUP: u32 = 1;
const G_BUFFER_BIND_GROUP: u32 = 2;

const WHITE: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);

// Camera
@group(CAMERA_BIND_GROUP) @binding(0)
var<uniform> camera: CameraUniform;

// Lights
@group(LIGHT_BIND_GROUP) @binding(0)
var<storage, read> lights: array<Light>;

@group(LIGHT_BIND_GROUP) @binding(1)
var<uniform> lights_len: u32;

// G-Buffer (action XD)
@group(G_BUFFER_BIND_GROUP) @binding(0)
var albedo_texture: texture_2d<f32>;

@group(G_BUFFER_BIND_GROUP) @binding(1)
var position_texture: texture_2d<f32>;

@group(G_BUFFER_BIND_GROUP) @binding(2)
var normal_texture: texture_2d<f32>;

@group(G_BUFFER_BIND_GROUP) @binding(3)
var material: texture_2d<f32>;

@group(G_BUFFER_BIND_GROUP) @binding(4)
var g_buffer_sampler: sampler;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;

    // Create a full-screen triangle
    let x = f32((vertex_index << 1) & 2);
    let y = f32(vertex_index & 2);
    output.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    output.uv = vec2<f32>(x, 1.0 - y);

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> FragmentOutput {
    var output: FragmentOutput;

    let albedo = textureSample(albedo_texture, g_buffer_sampler, input.uv);
    let normal = textureSample(normal_texture, g_buffer_sampler, input.uv);
    let position = textureSample(position_texture, g_buffer_sampler, input.uv);

    var result: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);

    let ambient_strength = 0.01;
    let ambient_color = WHITE * ambient_strength;

    for (var i: u32 = 0; i < lights_len; i++) {
        let light = lights[i];

        let pos = light.position;
        let color = light.color;
        let intensity = light.intensity;

        // TODO: use light direction instead
        let light_dir = normalize(pos.xyz - position.xyz);

        // Diffuse Lighting
        let diffuse_strength = max(dot(normal.xyz, light_dir), 0.0);
        let diffuse_color = color.rgb * diffuse_strength * intensity;

        result += (ambient_color + diffuse_color) * albedo.rgb;
    }

    output.color = vec4<f32>(result, 1.0);

    return output;
}
