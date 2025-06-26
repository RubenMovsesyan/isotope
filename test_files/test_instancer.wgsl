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
) {
    if (global_id.x < arrayLength(&instances)) {
        instances[global_id.x].position.x = 10.0 * cos(time * f32(global_id.x));
        instances[global_id.x].position.z = 10.0 * sin(time * f32(global_id.x));
        instances[global_id.x].position.y = 10.0 * sin(cos(time / f32(global_id.x)));
    }
}
