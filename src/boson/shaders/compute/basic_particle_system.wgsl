struct ModelInstance {
    position: vec3<f32>,
    orientation: vec4<f32>,
}

@group(0) @binding(0)
var<storage> time: f32;

@group(1) @binding(0)
var<storage, read_write> instances: array<ModelInstance>;

@compute @workgroup_size(256)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {}
