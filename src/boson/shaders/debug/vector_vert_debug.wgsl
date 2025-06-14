struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
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

@group(2) @binding(0)
var<storage> global_position: vec3<f32>;


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

// Vertex Shader
@vertex
fn main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;


    // Offset the point
    let world_position: vec4<f32> = vec4<f32>(model.position.xyz + global_position, 1.0);

    out.world_position = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;
    return out;
}
