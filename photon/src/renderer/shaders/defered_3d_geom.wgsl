struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
}

struct FragmentOutput {
    @location(0) albedo: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) material: vec4<f32>,
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

@vertex
fn vs_main(
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

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var output: FragmentOutput;

    output.albedo = vec4<f32>(1.0, 0.5, 0.3, 1.0);
    output.normal = vec4<f32>(normalize(in.normal) * 0.5 + 0.5, 1.0);
    output.material = vec4<f32>(0.5, 0.3, 1.0, 1.0);

    return output;
}
