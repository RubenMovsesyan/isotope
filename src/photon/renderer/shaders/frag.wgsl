struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
}

struct CameraUniform {
    view_position: vec4<f32>,
    view_proj: mat4x4<f32>,
};

struct Light {
    position: vec3<f32>,
    normal: vec3<f32>,
    color: vec3<f32>,
    intensity: f32
};

struct LightsBuffer {
    data: array<Light>,
}

struct Material {
    ambient_color: vec3<f32>,
    diffuse_color: vec3<f32>,
    specular_color: vec3<f32>,
    specular_focus: f32,
    optical_density: f32,
    dissolve: f32,
    illum: u32,
    optional_texture: u32,
}

// HACK: Fix this to be unbounded
const LIGHTS_UPPER_BOUND: u32 = 256;


// Camera
@group(0) @binding(0)
var<uniform> camera: CameraUniform;


// Lights
@group(1) @binding(0)
var<storage> lights: array<Light, LIGHTS_UPPER_BOUND>;

@group(1) @binding(1)
var<uniform> lights_len: u32;


// Material
@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(2) @binding(1)
var s_diffuse: sampler;

@group(4) @binding(0)
var<storage> material: Material;

const WHITE: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);

// Fragment shader
@fragment
fn main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    var object_color: vec4<f32>;

    if (material.optional_texture == 1) {
        object_color = textureSample(t_diffuse, s_diffuse, in.uv_coords);
    } else {
        object_color = vec4<f32>(material.diffuse_color.rbg, 1.0); // for some reason the color is in rbg
    }

    // object_color += vec4<f32>(material.ambient_color, 1.0);
    // object_color /= 2.0;


    var result: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);

    // Ambient lighting
    let ambient_strength = 0.01;
    let ambient_color = WHITE * ambient_strength * material.ambient_color;

    if (lights_len == 0) {
        result += ambient_color * object_color.rgb;
    }

    for (var i: u32 = 0; i < lights_len; i++) {
        let pos = lights[i].position;
        let color = lights[i].color;
        let intensity = lights[i].intensity;

        // TODO: use light dir instead
        let light_dir = normalize(pos.xyz - in.world_position);

        // Diffuse Lighting
        let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
        let diffuse_color = color.rgb * diffuse_strength * intensity;

        // Specular Lighting
        let view_dir = normalize(camera.view_position.xyz - in.world_position);
        let reflect_dir = reflect(-light_dir, in.world_normal);
        let specular_strength = pow(max(dot(view_dir, reflect_dir), 0.0), material.specular_focus);
        let specular_color = specular_strength * color.rgb * material.specular_color.rgb;

        result += (ambient_color + diffuse_color + specular_color) * object_color.rgb;
    }

    return vec4<f32>(result, object_color.a);
}
