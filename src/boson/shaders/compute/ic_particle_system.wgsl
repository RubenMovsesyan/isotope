struct ModelInstance {
    position: vec3<f32>,
    orientation: vec4<f32>,
}

struct InitialCondition {
    position: vec3<f32>,
    velocity: vec3<f32>,
}

@group(0) @binding(0)
var<storage> delta_t: f32;

@group(0) @binding(1)
var<storage> initial_conditions: array<InitialCondition>;

@group(0) @binding(2)
var<storage, read_write> reset: u32;

@group(0) @binding(3)
var<storage, read_write> velocity: array<vec3<f32>>;

@group(1) @binding(0)
var<storage, read_write> instances: array<ModelInstance>;

@compute @workgroup_size(256)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    let index = global_id.x;

    if (index < arrayLength(&instances)) {
        // Reset if the reset signal has been sent
        if (reset == 1) {
            velocity[index] = initial_conditions[index].velocity;
            instances[index].position = initial_conditions[index].position;
            reset = 0;
        }

        // Basic Kinematics otherwise
        // TEMP GRAVITY IS JUST IN THE DOWN DIRECTION
        velocity[index] += vec3<f32>(0.0, -9.81, 0.0) * delta_t;

        instances[index].position += velocity[index] * delta_t;
    }
}
