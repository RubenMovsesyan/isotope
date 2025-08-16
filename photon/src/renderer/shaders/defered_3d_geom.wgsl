struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) debug_clip_pos: vec4<f32>,
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
    // out.uv_coords = model.uv_coords;
    // out.world_normal = normalize(model.normal);

    // let combined_rotation = quat_norm(hamilton_prod(global_transform.rotation, instance.rotation));

    // Rotate the point first
    // let conj = quat_conj(combined_rotation);
    // let conj = quat_conj(instance.rotation);

    // let rot_normal: vec4<f32> = hamilton_prod(
    //     hamilton_prod(
    //         // combined_rotation,
    //         instance.rotation,
    //         vec4<f32>(model.normal, 0.0),
    //     ),
    //     conj
    // );
    // out.world_normal = normalize(rot_normal.xyz);

    // let rot: vec4<f32> = hamilton_prod(
    //     hamilton_prod(
    //         // combined_rotation,
    //         instance.rotation,
    //         vec4<f32>(model.position, 0.0),
    //     ),
    //     conj
    // );

    // Offset the point
    // let world_position: vec4<f32> = vec4<f32>(rot.xyz + instance.position + global_transform.position, 1.0);
    // let world_position: vec4<f32> = vec4<f32>(model.position, 1.0);
    // out.world_position = world_position.xyz;
    // out.clip_position = camera.view_proj * world_position;

    // out.debug_clip_pos = out.clip_position;

    // Debug: Output raw vertex position without any transformation
    // This will tell us if we're getting valid vertex data
    // out.clip_position = vec4<f32>(model.position.xy, 0.5, 1.0);
    let world_position = vec4<f32>(model.position + instance.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;

    // Debug: Also pass clip position for visualization
    out.uv_coords = vec2<f32>(
        out.clip_position.x / out.clip_position.w,
        out.clip_position.y / out.clip_position.w
    );
    out.world_normal = model.normal;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var output: FragmentOutput;

    // Now use the interpolated clip position from vertices
    // let screen_x = in.debug_clip_pos.x / in.debug_clip_pos.w;
    // let screen_y = in.debug_clip_pos.y / in.debug_clip_pos.w;
    // let screen_z = in.debug_clip_pos.z / in.debug_clip_pos.w;

    // // Debug: Show if we're inside normal clip space
    // if (abs(screen_x) > 1.0 || abs(screen_y) > 1.0) {
    //     // Outside clip space - show red
    //     output.albedo = vec4<f32>(1.0, 0.0, 0.0, 1.0);
    // } else {
    //     // Inside clip space - show position as color
    //     output.albedo = vec4<f32>(
    //         (screen_x + 1.0) * 0.5,
    //         (screen_y + 1.0) * 0.5,
    //         screen_z,
    //         1.0
    //     );
    // }


    // Show where we are in clip space
    let clip_x = in.uv_coords.x;
    let clip_y = in.uv_coords.y;

    // Red channel: how far right (clip_x > 0)
    // Green channel: how far up (clip_y > 0)
    // Blue channel: if we're outside normal clip space
    let outside = f32(abs(clip_x) > 1.0 || abs(clip_y) > 1.0);

    // output.albedo = vec4<f32>(
    //     saturate(clip_x + 0.5),
    //     saturate(clip_y + 0.5),
    //     outside,
    //     1.0
    // );
    output.albedo = vec4<f32>(in.clip_position.xy, 0.0, 1.0);

    output.normal = vec4<f32>(0.5, 0.5, 1.0, 1.0);
    output.material = vec4<f32>(0.5, 0.3, 1.0, 1.0);

    return output;
}
