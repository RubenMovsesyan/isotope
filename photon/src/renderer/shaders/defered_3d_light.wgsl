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

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var albedo_texture: texture_2d<f32>;

@group(1) @binding(1)
var normal_texture: texture_2d<f32>;

@group(1) @binding(2)
var material: texture_2d<f32>;

@group(1) @binding(3)
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

    // Debug: Show a checkerboard pattern where geometry exists
    // let checker = (u32(input.uv.x * 10.0) + u32(input.uv.y * 10.0)) % 2u;

    // if (albedo.a > 0.01) {
    //     // Geometry exists here - show albedo with checkerboard
    //     output.color = mix(albedo, vec4<f32>(0.0, 1.0, 0.0, 1.0), f32(checker) * 0.5);
    // } else {
    //     // No geometry - show black/blue gradient based on UV
    //     output.color = vec4<f32>(input.uv.x, input.uv.y, 0.0, 1.0);
    // }

    // output.color = vec4<f32>(albedo.a, albedo.a, albedo.a, 1.0);
    output.color = vec4<f32>(albedo.rgb, 1.0);
    // output.color = vec4<f32>(input.uv, 0.0, 1.0);

    return output;
}
