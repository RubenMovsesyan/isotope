struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
}

struct FragmentOutput {
    @location(0) albedo: vec4<f32>,
    @location(1) position: vec4<f32>,
    @location(2) normal: vec4<f32>,
    @location(3) material: vec4<f32>,
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

// Bind Groups
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

fn hamilton_prod(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(
        a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
        a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
        a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
        a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
    );
}

fn quat_conj(q: vec4<f32>) -> vec4<f32> {
    return q * vec4<f32>(-1.0, -1.0, -1.0, 1.0);
}

fn quat_norm(q: vec4<f32>) -> vec4<f32> {
    return q / length(q);
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
    @builtin(vertex_index) vertex_idx: u32,
) -> VertexOutput {
var out: VertexOutput;
    let world_position = vec4<f32>(model.position + instance.position, 1.0);

    out.world_position = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;
    out.world_normal = model.normal;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var output: FragmentOutput;

    // Color of the object
    output.albedo = vec4<f32>(1.0, 0.0, 1.0, 1.0);

    // Position of the fragment
    output.position = vec4<f32>(in.world_position, 1.0);

    // Normals of the object
    output.normal = vec4<f32>(in.world_normal, 1.0);

    output.material = vec4<f32>(0.5, 0.3, 1.0, 1.0);

    return output;
}
