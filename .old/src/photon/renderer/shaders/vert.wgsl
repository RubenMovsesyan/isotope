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

struct GlobalTransform {
    position: vec3<f32>,
    rotation: vec4<f32>,
}

struct CameraUniform {
    view_position: vec4<f32>,
    view_proj: mat4x4<f32>,
}

// Bind Groups
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(3) @binding(0)
var<storage> global_transform: GlobalTransform;


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
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv_coords = model.uv_coords;

    let combined_rotation = quat_norm(hamilton_prod(global_transform.rotation, instance.rotation));

    // Rotate the point first
    let conj = quat_conj(combined_rotation);

    let rot_normal: vec4<f32> = hamilton_prod(
        hamilton_prod(
            combined_rotation,
            vec4<f32>(model.normal, 0.0),
        ),
        conj
    );
    out.world_normal = normalize(rot_normal.xyz);

    let rot: vec4<f32> = hamilton_prod(
        hamilton_prod(
            combined_rotation,
            vec4<f32>(model.position, 0.0),
        ),
        conj
    );

    // Offset the point
    let world_position: vec4<f32> = vec4<f32>(rot.xyz + instance.position + global_transform.position, 1.0);

    out.world_position = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;
    return out;
}
