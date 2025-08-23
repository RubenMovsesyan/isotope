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
var normal_texture: texture_2d<f32>;

@group(G_BUFFER_BIND_GROUP) @binding(2)
var material: texture_2d<f32>;

@group(G_BUFFER_BIND_GROUP) @binding(3)
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

    let view_dir = normalize(vec3<f32>(0.0, 0.0, 1.0));

    let n_dot_v = max(dot(normal.xyz, view_dir), 0.0);

    let ambient = 0.2;
    let diffuse = 0.8 * n_dot_v;
    let lighting = ambient + diffuse;

    output.color = vec4<f32>(albedo.rgb * lighting, 1.0);

    return output;
}
