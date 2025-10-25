struct InstanceInput {
    position: vec3<f32>,
    rotation: vec4<f32>,
}

@group(0) @binding(0)
var<storage, read_write> instances: array<InstanceInput>;

@group(0) @binding(1)
var<uniform> delta_t: f32;

@group(0) @binding(2)
var<uniform> t: f32;

@compute @workgroup_size(256)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    let index = global_id.x;

    instances[index].position += vec3<f32>(sin(t), cos(t), 0.0) * 0.01;
}
