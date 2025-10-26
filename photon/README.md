# Photon - Deferred Rendering Engine

A high-performance deferred rendering system for the Isotope game engine. Photon provides GPU-accelerated 3D rendering with support for multiple dynamic lights, advanced materials, and efficient camera systems.

## Overview

Photon is the rendering subsystem of Isotope, handling all GPU-based graphics operations. It implements a deferred rendering pipeline optimized for scenes with many dynamic lights.

**Key Features:**
- **Deferred Rendering Pipeline**: Efficient multi-light rendering
- **3D Camera System**: Perspective and orthographic projections with frustum culling
- **Dynamic Lighting**: Support for multiple point lights with customizable properties
- **Material System**: Texture-based materials with lighting calculations
- **GPU Shaders**: WGSL compute and fragment shaders for modern GPU acceleration
- **Texture Management**: Efficient texture loading and binding
- **Frame Buffer Management**: G-buffer based deferred pipeline

## Architecture

### Rendering Pipeline

```
┌─────────────────────────────────────────┐
│         Geometry Pass (G-Buffer)        │
│  Render all meshes to intermediate      │
│  targets: positions, normals, colors    │
├─────────────────────────────────────────┤
│         Lighting Pass                   │
│  Process accumulated lights using       │
│  G-buffer data                          │
├─────────────────────────────────────────┤
│         Composition Pass                │
│  Combine results to screen              │
├─────────────────────────────────────────┤
│         Output to Framebuffer           │
│  Present to window                      │
└─────────────────────────────────────────┘
```

### Deferred Rendering Benefits

**Traditional Forward Rendering:**
- Cost: O(objects × lights)
- 10 objects × 100 lights = 1000 calculations

**Deferred Rendering:**
- Cost: O(objects) + O(lights)
- 10 objects + 100 lights = 110 calculations

Deferred rendering scales much better with light count, making it ideal for dynamic-heavy scenes.

### Data Flow

```
Models & Transforms
        ↓
Camera Projection
        ↓
Geometry Pass (G-Buffer)
        ↓
    ┌───┴────────┬──────────┐
    ↓            ↓          ↓
 Position    Normals    Albedo
    └───┬────────┴──────────┘
        ↓
Lighting Pass
        ↓
   ┌────┴────┐
   ↓         ↓
 Lights  G-Buffer Data
   └────┬────┘
        ↓
 Final Composite
        ↓
   Screen Output
```

## Core Components

### Renderer

The main rendering system coordinating all rendering operations:

```rust
pub struct Renderer {
    gpu_controller: Arc<GpuController>,
    camera_layout: BindGroupLayout,
    material_layout: BindGroupLayout,
    geometry_pipeline: RenderPipeline,
    lighting_pipeline: RenderPipeline,
}

impl Renderer {
    pub fn new(gpu_controller: Arc<GpuController>) -> Result<Self>
    pub fn render(&self, ecs: &Compound) -> Result<()>
    pub fn submit_frame(&self) -> Result<()>
}
```

### Camera System

Support for multiple camera types with projection matrices:

```rust
pub struct Camera3D {
    projection: CameraProjection,
    view_matrix: Matrix4<f32>,
    position: Vector3<f32>,
    target: Vector3<f32>,
}

#[derive(Clone)]
pub enum CameraProjection {
    Perspective {
        fov: f32,           // Field of view in degrees
        aspect: f32,        // Width/height ratio
        near: f32,          // Near clipping plane
        far: f32,           // Far clipping plane
    },
    Orthographic {
        width: f32,
        height: f32,
        near: f32,
        far: f32,
    },
}

impl Camera3D {
    pub fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Self
    pub fn orthographic(width: f32, height: f32, near: f32, far: f32) -> Self
    
    pub fn get_view_matrix(&self) -> Matrix4<f32>
    pub fn get_projection_matrix(&self) -> Matrix4<f32>
    pub fn get_frustum(&self) -> Frustum
}
```

#### Perspective Camera

Realistic 3D projection mimicking human vision:

```rust
let camera = Camera3D::perspective(
    60.0,                      // 60-degree field of view
    1920.0 / 1080.0,          // Aspect ratio
    0.1,                       // Near plane
    1000.0                     // Far plane
);
```

**Use Cases:**
- First-person games
- Third-person games
- Realistic 3D viewing

#### Orthographic Camera

Parallel projection for 2D-like rendering:

```rust
let camera = Camera3D::orthographic(
    1920.0,                    // Width in world units
    1080.0,                    // Height in world units
    -100.0,                    // Near plane
    100.0                      // Far plane
);
```

**Use Cases:**
- 2D games
- UI rendering
- Isometric views
- Strategy games

### Frustum Culling

Automatically cull objects outside the camera's view:

```rust
pub struct Frustum {
    planes: [Plane; 6],  // Near, far, left, right, top, bottom
}

impl Frustum {
    pub fn contains_point(&self, point: Vector3<f32>) -> bool
    pub fn contains_sphere(&self, center: Vector3<f32>, radius: f32) -> bool
    pub fn contains_aabb(&self, min: Vector3<f32>, max: Vector3<f32>) -> bool
}
```

Usage:
```rust
let frustum = camera.get_frustum();

if frustum.contains_sphere(object_pos, object_radius) {
    render_object(object);
}
```

### Lighting System

Dynamic lighting with multiple light types:

```rust
pub struct Light {
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
}

impl Light {
    pub fn new(
        position: [f32; 3],
        direction: [f32; 3],
        color: [f32; 3],
        intensity: f32,
    ) -> Self

    pub fn set_position(&mut self, pos: [f32; 3])
    pub fn set_color(&mut self, color: [f32; 3])
    pub fn set_intensity(&mut self, intensity: f32)
}
```

#### Point Lights

Lights that emit in all directions:

```rust
let light = Light::new(
    [10.0, 5.0, 10.0],    // Position
    [0.0, 0.0, 0.0],      // Direction (unused for point lights)
    [1.0, 1.0, 1.0],      // White color
    1.0                     // Intensity
);
```

#### Directional Lights

Lights that emit from a direction (like sunlight):

```rust
let sunlight = Light::new(
    [0.0, 0.0, 0.0],                    // Position (relative)
    [0.0, -1.0, 0.0],                   // Direction (downward)
    [1.0, 0.95, 0.8],                   // Warm white
    2.0                                  // Bright
);
```

### Material System

Materials define visual appearance:

```rust
pub struct Material {
    pub name: String,
    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub shininess: f32,
    pub texture: Option<Arc<Texture>>,
}

impl Material {
    pub fn new(name: &str) -> Self
    pub fn set_color(&mut self, color: [f32; 3])
    pub fn set_texture(&mut self, texture: Arc<Texture>)
    pub fn with_shininess(mut self, shininess: f32) -> Self
}
```

#### Material Properties

- **Ambient**: How the material appears in shadow
- **Diffuse**: Base color of the material
- **Specular**: How reflective the material is
- **Shininess**: How sharp reflections are
- **Texture**: Texture map for detail

#### Creating Materials

```rust
let wood = Material::new("Wood")
    .with_ambient([0.2, 0.1, 0.05])
    .with_diffuse([0.8, 0.4, 0.2])
    .with_specular([0.3, 0.3, 0.3])
    .with_shininess(32.0);

let metal = Material::new("Metal")
    .with_diffuse([0.5, 0.5, 0.5])
    .with_specular([1.0, 1.0, 1.0])
    .with_shininess(256.0);
```

### Texture Management

Efficient texture loading and caching:

```rust
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub data: Vec<u8>,
}

impl Texture {
    pub fn from_image(
        path: &str,
        assets: &AssetServer,
    ) -> Result<Arc<Self>>

    pub fn from_raw(
        width: u32,
        height: u32,
        format: TextureFormat,
        data: Vec<u8>,
    ) -> Self
}
```

#### Supported Formats

- **PNG** - Lossless compression
- **JPG** - Lossy compression (good for photographs)
- **BMP** - Uncompressed (fast loading)

#### Texture Properties

```rust
let texture = Texture::from_image("assets/wood.png", assets)?;
println!("Texture: {}x{}", texture.width, texture.height);
```

## Shaders

Photon uses WGSL (WebGPU Shading Language) for GPU programs.

### Geometry Shader

Renders geometry to G-buffers (positions, normals, colors):

```wgsl
struct CameraData {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    position: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraData;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let world_pos = vec3<f32>(input.position);
    let clip_pos = camera.projection * camera.view * vec4<f32>(world_pos, 1.0);
    
    return VertexOutput(
        clip_pos,
        world_pos,
        input.normal,
        input.uv,
    );
}
```

### Lighting Shader

Calculates lighting using G-buffer data:

```wgsl
struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
    intensity: f32,
}

@compute @workgroup_size(16, 16)
fn cs_light(
    @builtin(global_invocation_id) id: vec3<u32>,
) {
    let pixel = id.xy;
    let g_pos = textureLoad(g_position, pixel, 0);
    let g_normal = textureLoad(g_normal, pixel, 0);
    
    var lighting = vec3<f32>(0.0);
    
    for (var i: u32 = 0u; i < num_lights; i = i + 1u) {
        let light = lights[i];
        let to_light = normalize(light.position - g_pos.xyz);
        let diffuse = max(dot(g_normal.xyz, to_light), 0.0);
        lighting = lighting + light.color * diffuse * light.intensity;
    }
    
    textureStore(output, pixel, vec4<f32>(lighting, 1.0));
}
```

## Usage Examples

### Setting Up the Renderer

```rust
use photon::renderer::Renderer;
use gpu_controller::GpuController;
use std::sync::Arc;

let gpu = Arc::new(GpuController::new(None, None, None).await?);
let renderer = Renderer::new(gpu)?;
```

### Creating Cameras

```rust
use photon::camera::Camera3D;
use isotope::Transform3D;
use cgmath::{Vector3, Quaternion};

// Perspective camera
let camera = Camera3D::perspective(60.0, 16.0 / 9.0, 0.1, 1000.0);

// Spawn in ECS with transform
ecs.spawn((
    camera,
    Transform3D::new(
        Vector3::new(0.0, 5.0, 10.0),
        Quaternion::one(),
    ),
));
```

### Adding Lights

```rust
use photon::Light;

// Create a warm light
let light = Light::new(
    [5.0, 3.0, 5.0],       // Position
    [0.0, 0.0, 0.0],       // Direction
    [1.0, 0.9, 0.7],       // Warm white
    2.0                     // Intensity
);

ecs.spawn((light,));

// Update light position
ecs.iter_mut_mol(|_, light: &mut Light| {
    light.set_position([
        10.0 * f32::cos(t),
        5.0,
        10.0 * f32::sin(t),
    ]);
});
```

### Material Creation

```rust
use photon::renderer::Material;

let brick = Material::new("Brick")
    .with_diffuse([0.7, 0.3, 0.2])
    .with_specular([0.2, 0.2, 0.2])
    .with_shininess(16.0);

let steel = Material::new("Steel")
    .with_diffuse([0.5, 0.5, 0.5])
    .with_specular([1.0, 1.0, 1.0])
    .with_shininess(256.0);
```

### Complete Rendering Example

```rust
use isotope::*;
use winit::event_loop::EventLoop;

#[derive(Default)]
struct RenderTest;

impl IsotopeState for RenderTest {
    fn init(&mut self, ecs: &Compound, assets: &AssetServer) {
        // Create camera
        ecs.spawn((
            Camera::perspective_3d_default(assets),
            Transform3D::new(
                Vector3::new(0.0, 5.0, 15.0),
                Quaternion::from_axis_angle(Vector3::unit_y(), Deg(0.0)),
            ),
        ));

        // Load model
        match Model::from_obj("assets/model.obj", assets, None) {
            Ok(model) => {
                ecs.spawn((
                    model,
                    Transform3D::default(),
                ));
            }
            Err(e) => eprintln!("Failed to load model: {}", e),
        }

        // Add lighting
        ecs.spawn((
            Light::new(
                [10.0, 5.0, 10.0],
                [0.0, 0.0, 0.0],
                [1.0, 1.0, 1.0],
                2.0,
            ),
        ));

        ecs.spawn((
            Light::new(
                [-10.0, 3.0, -10.0],
                [0.0, 0.0, 0.0],
                [0.5, 0.5, 1.0],  // Blue
                1.0,
            ),
        ));
    }

    fn update(&mut self, ecs: &Compound, _: &AssetServer, _: f32, t: f32) {
        // Rotate model
        ecs.iter_mut_mol(|_, transform: &mut Transform3D| {
            transform.rotation = Quaternion::from_axis_angle(
                Vector3::unit_y(),
                Deg(t * 45.0),
            );
        });

        // Animate lights
        ecs.iter_mut_mol(|_, light: &mut Light| {
            light.set_position([
                10.0 * f32::cos(t),
                5.0,
                10.0 * f32::sin(t),
            ]);
        });
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let isotope = Isotope::<RenderTest>::new(
        &event_loop,
        "Photon Test",
        1920,
        1080,
    ).unwrap();

    event_loop.run_app(&mut isotope.into()).unwrap();
}
```

## Performance Optimization

### Frustum Culling

Skip rendering objects outside camera view:

```rust
let frustum = camera.get_frustum();

ecs.iter_mol(|_, (transform, model): (&Transform3D, &Model)| {
    let bounds = model.get_bounds();
    if frustum.contains_aabb(bounds.min, bounds.max) {
        render_model(model, transform);
    }
});
```

### LOD (Level of Detail)

Use simpler meshes for distant objects:

```rust
let distance = camera.position.distance(object.position);

if distance > 100.0 {
    render_lod_2(object);  // Simple mesh
} else if distance > 50.0 {
    render_lod_1(object);  // Medium mesh
} else {
    render_lod_0(object);  // High detail
}
```

### Batching

Reduce state changes by grouping similar objects:

```rust
// Group by material
let mut by_material = HashMap::new();
ecs.iter_mol(|_, (transform, model): (&Transform3D, &Model)| {
    for material in &model.materials {
        by_material.entry(material.name.clone())
            .or_insert_with(Vec::new)
            .push((transform, model));
    }
});

// Render by material (fewer state changes)
for (material, objects) in by_material {
    set_material(&material);
    for (transform, model) in objects {
        render_model(model, transform);
    }
}
```

### Light Optimization

Limit number of active lights:

```rust
// Only render closest N lights
let mut lights: Vec<_> = ecs.iter_mol(|_, light: &Light| {
    (camera.position.distance(light.position.into()), light)
}).collect();

lights.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
lights.truncate(16);  // Max 16 lights

for (_, light) in lights {
    render_light_effect(light);
}
```

## Advanced Features

### Shadow Mapping

Implement shadows using depth maps:

```rust
// Render scene from light's perspective
let light_pass = RenderPass::new()
    .with_target(shadow_depth_texture)
    .render_geometry(ecs);

// Use depth map during lighting pass
let lighting_pass = RenderPass::new()
    .with_target(output)
    .use_shadow_map(shadow_depth_texture)
    .render_lights(ecs);
```

### Post-Processing

Apply effects after rendering:

```rust
pub struct PostProcessing {
    bloom: bool,
    motion_blur: bool,
    color_grading: bool,
}

impl PostProcessing {
    pub fn apply(&self, input: Texture) -> Texture {
        let mut result = input;
        
        if self.bloom {
            result = apply_bloom(&result);
        }
        if self.motion_blur {
            result = apply_motion_blur(&result);
        }
        if self.color_grading {
            result = apply_color_grading(&result);
        }
        
        result
    }
}
```

### MSAA (Anti-Aliasing)

Enable multisample anti-aliasing:

```rust
let render_pass = RenderPassDescriptor {
    label: Some("Main Pass"),
    color_attachments: &[Some(RenderPassColorAttachment {
        view: &frame.texture.create_view(&Default::default()),
        resolve_target: None,
        ops: Operations {
            load: LoadOp::Clear(Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }),
            store: StoreOp::Store,
        },
    })],
    depth_stencil_attachment: None,
    timestamp_writes: None,
    occlusion_query_set: None,
};
```

## G-Buffer Layout

The deferred rendering pipeline uses multiple render targets:

### G-Buffer 0: Positions
- Stores world-space positions
- Format: R32G32B32A32 Float

### G-Buffer 1: Normals
- Stores surface normals
- Format: R32G32B32A32 Float (compressed)

### G-Buffer 2: Albedo
- Stores base colors
- Format: R8G8B8A8 UNORM

### G-Buffer 3: PBR Properties
- Roughness, metallic, ambient occlusion
- Format: R8G8B8A8 UNORM

## Lighting Calculations

### Lambertian Diffuse

Simple diffuse lighting model:

```glsl
float diffuse = max(dot(normal, to_light), 0.0);
vec3 diffuse_color = material.diffuse * light.color * diffuse;
```

### Blinn-Phong Specular

Specular highlights:

```glsl
vec3 to_camera = normalize(camera_pos - world_pos);
vec3 half_vector = normalize(to_light + to_camera);
float spec_angle = max(dot(normal, half_vector), 0.0);
float specular = pow(spec_angle, shininess);
vec3 specular_color = material.specular * light.color * specular;
```

### Combined Lighting

```glsl
vec3 lighting = material.ambient +
                diffuse_color +
                specular_color;
```

## Dependencies

- **gpu_controller** - GPU resource management
- **matter_vault** - Shared data containers
- **cgmath** (0.18.0) - 3D mathematics
- **bytemuck** (1.23.1) - Memory manipulation
- **log** (0.4.27) - Logging
- **wgpu** (0.20+) - GPU abstraction
- **anyhow** - Error handling

## Debugging

Enable rendering debug logs:

```bash
RUST_LOG=photon=debug cargo run
```

Check for common issues:
- Shader compilation errors
- Missing textures
- Incorrect material properties
- Camera clipping

## Troubleshooting

### Objects not rendering

1. **Check camera frustum** - Is object within camera's view?
2. **Verify transforms** - Are position/rotation correct?
3. **Check materials** - Are materials assigned?
4. **Enable backface rendering** - Some models have reversed normals

### Lighting issues

1. **Check light position** - Is light near scene?
2. **Verify light color** - Is light color non-black?
3. **Check intensity** - Is intensity high enough?
4. **Verify normals** - Are surface normals correct?

### Performance issues

1. **Profile with perf** - Identify bottleneck
2. **Reduce light count** - Limit active lights
3. **Enable frustum culling** - Skip off-screen objects
4. **Use LOD systems** - Reduce geometry for distant objects

## Future Enhancements

- [ ] Screen-space ambient occlusion (SSAO)
- [ ] Screen-space reflections (SSR)
- [ ] Real-time ray tracing
- [ ] Global illumination
- [ ] Volumetric lighting
- [ ] Physically-based rendering (PBR)
- [ ] Advanced shadow techniques
- [ ] Particle lighting integration

## See Also

- [Main Isotope Documentation](../README.md)
- [GPU Controller](../gpu_controller/README.md)
- [Boson Physics](../boson/README.md)
- [Compound ECS](../compound/README.md)