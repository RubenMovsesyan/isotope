# Isotope - Core Game Engine Framework

The main integration layer of the Isotope game engine, bringing together rendering, physics, ECS, and asset management into a cohesive, easy-to-use framework for building interactive 3D applications.

## Overview

Isotope is the heart of the engine workspace. It coordinates between all subsystems and provides a high-level API for game developers:

- **Application Framework**: Event-driven application loop
- **Window Management**: Winit-based window and input handling
- **ECS Integration**: Seamless Compound ECS integration
- **Physics Integration**: Boson physics engine connection
- **Rendering Pipeline**: Photon deferred rendering system
- **Asset Management**: Centralized asset loading and caching
- **Component System**: Pre-built components for common game needs

## Architecture

### Layered Integration

```
Application Code
        ↓
   IsotopeState
        ↓
   Isotope Framework
    ↙    ↓    ↘
Compound  Photon  Boson  GPU Controller
(ECS)   (Render) (Physics)
    ↘    ↓    ↙
   WGPU & Winit
        ↓
   GPU Hardware
```

### Core Components

```rust
pub struct Isotope<T: IsotopeState> {
    // GPU and rendering
    gpu_controller: Arc<GpuController>,
    renderer: Arc<Renderer>,
    
    // Game logic
    ecs: Arc<Compound>,
    physics: Arc<Boson>,
    
    // Resources
    asset_server: Arc<AssetServer>,
    
    // State management
    state: T,
    
    // Window and events
    window: Option<Arc<RenderingWindow>>,
    
    // Timing
    last_frame_time: Instant,
    tick_rate: Duration,
}
```

## Getting Started

### Creating Your First Application

1. **Define your game state**

```rust
use isotope::*;

#[derive(Default)]
struct MyGame {
    camera_speed: f32,
    // Your game state here
}

impl IsotopeState for MyGame {
    fn init(&mut self, ecs: &Compound, assets: &AssetServer) {
        // Called once at startup
        // Load assets, spawn initial entities
    }

    fn update(&mut self, ecs: &Compound, assets: &AssetServer, delta_t: f32, t: f32) {
        // Called every frame
        // Update game logic
    }

    fn render(&mut self, _ecs: &Compound) {
        // Optional: custom rendering logic
        // Usually handled automatically
    }

    fn handle_event(&mut self, event: &WindowEvent) -> bool {
        // Handle window events
        // Return true to consume the event
        false
    }
}
```

2. **Initialize the engine**

```rust
use winit::event_loop::EventLoop;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let isotope = Isotope::<MyGame>::new(
        &event_loop,
        "My Game",
        1920,
        1080,
    ).unwrap();
    
    event_loop.run_app(&mut isotope.into()).unwrap();
}
```

## Core Systems

### Transform System

Every visual entity can have a `Transform3D` component for positioning, rotating, and scaling:

```rust
pub struct Transform3D {
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
    scale: Vector3<f32>,
}

impl Transform3D {
    pub fn new(position: Vector3<f32>, rotation: Quaternion<f32>) -> Self
    pub fn translate(&mut self, offset: Vector3<f32>)
    pub fn rotate(&mut self, rotation: Quaternion<f32>)
    pub fn set_scale(&mut self, scale: Vector3<f32>)
}
```

Usage:

```rust
ecs.spawn((
    model,
    Transform3D::new(
        Vector3::new(0.0, 0.0, 0.0),
        Quaternion::one(),
    ),
));

// Update transforms
ecs.iter_mut_mol(|_, transform: &mut Transform3D| {
    transform.translate(Vector3::new(1.0, 0.0, 0.0));
});
```

### Model and Mesh System

Load and render 3D models from OBJ files:

```rust
pub struct Model {
    meshes: Vec<Mesh>,
    materials: Vec<Material>,
}

impl Model {
    pub fn from_obj(
        path: &str,
        assets: &AssetServer,
        instances: Option<&[Instance]>,
    ) -> Result<Self>
}
```

Usage:

```rust
let model = Model::from_obj("assets/model.obj", assets, None)?;
ecs.spawn((
    model,
    Transform3D::default(),
));
```

### Material System

Control visual appearance with materials and textures:

```rust
pub struct Material {
    name: String,
    ambient: [f32; 3],
    diffuse: [f32; 3],
    specular: [f32; 3],
    shininess: f32,
    texture: Option<Arc<Texture>>,
}

impl Material {
    pub fn new(name: &str) -> Self
    pub fn set_color(&mut self, color: [f32; 3])
    pub fn set_texture(&mut self, texture: Arc<Texture>)
}
```

### Texture System

Load and manage textures efficiently:

```rust
pub struct Texture {
    width: u32,
    height: u32,
    format: TextureFormat,
    data: Vec<u8>,
}

impl Texture {
    pub fn from_image(
        path: &str,
        assets: &AssetServer,
    ) -> Result<Arc<Self>>
}
```

### Camera System

Multiple camera types for different perspectives:

```rust
pub struct Camera {
    projection: CameraProjection,
    transform: Transform3D,
}

#[derive(Clone)]
pub enum CameraProjection {
    Perspective { fov: f32, aspect: f32, near: f32, far: f32 },
    Orthographic { width: f32, height: f32, near: f32, far: f32 },
}

impl Camera {
    pub fn perspective_3d_default(assets: &AssetServer) -> Self
    pub fn orthographic_2d(width: f32, height: f32) -> Self
}
```

Usage:

```rust
ecs.spawn((
    Camera::perspective_3d_default(assets),
    Transform3D::new(
        Vector3::new(0.0, 5.0, 10.0),
        Quaternion::from_axis_angle(Vector3::unit_y(), Deg(0.0)),
    ),
));
```

### Instancer System

Render many copies of the same model efficiently using GPU instancing:

```rust
pub struct Instancer {
    instances: Vec<Instance>,
    update_fn: Option<Box<dyn Fn(&mut [Instance], f32, f32)>>,
}

impl Instancer {
    pub fn new_parallel(
        range: Option<Range<usize>>,
        assets: &AssetServer,
        initial: Vec<Instance>,
        shader: &str,
    ) -> Self

    pub fn new_serial<F>(
        range: Option<Range<usize>>,
        update: F,
    ) -> Self
    where
        F: Fn(&mut [Instance], f32, f32) + 'static,
}
```

Usage:

```rust
let instances = vec![
    Instance::new(Vector3::new(0.0, 0.0, 0.0), Quaternion::one(), Matrix4::identity()),
    Instance::new(Vector3::new(5.0, 0.0, 0.0), Quaternion::one(), Matrix4::identity()),
];

ecs.spawn((
    model,
    Instancer::new_serial(Some(0..2), |instances, delta_t, t| {
        for instance in instances {
            instance.pos(|pos| {
                pos.y = f32::sin(t) * 5.0;
            });
        }
    }),
));
```

### Physics Components

Integrate physics with the ECS:

```rust
pub use boson::{PointMass, RigidBody, StaticCollider};

// Spawn physics objects
ecs.spawn((
    Transform3D::default(),
    PointMass::new(10.0),  // 10 kg particle
));

// Physics and transform stay in sync
ecs.iter_mut_duo(|_, transform: &mut Transform3D, physics: &PointMass| {
    // Update visual position from physics
});
```

### Lighting

Create and manage dynamic lights:

```rust
pub struct Light {
    position: [f32; 3],
    direction: [f32; 3],
    color: [f32; 3],
    intensity: f32,
}

impl Light {
    pub fn new(
        position: [f32; 3],
        direction: [f32; 3],
        color: [f32; 3],
        intensity: f32,
    ) -> Self
}
```

Usage:

```rust
ecs.spawn((
    Light::new(
        [10.0, 5.0, 10.0],
        [0.0, 0.0, 0.0],
        [1.0, 1.0, 1.0],
        1.0,
    ),
));

// Update lights each frame
ecs.iter_mut_mol(|_, light: &mut Light| {
    light.pos(|pos| {
        pos[0] = 10.0 * f32::cos(t);
        pos[2] = 10.0 * f32::sin(t);
    });
});
```

## Asset Server

Centralized asset loading and caching:

```rust
pub struct AssetServer {
    assets: Arc<RwLock<HashMap<String, Arc<dyn Any>>>>,
}

impl AssetServer {
    pub fn new() -> Self
    
    pub fn load_asset<T: 'static>(&self, path: &str) -> Result<Arc<T>>
    pub fn get_asset<T: 'static>(&self, path: &str) -> Option<Arc<T>>
    pub fn cache_asset<T: 'static>(&self, path: String, asset: Arc<T>)
}
```

Usage:

```rust
let assets = AssetServer::new();

// Load and cache
let texture = assets.load_asset::<Texture>("assets/texture.png")?;

// Retrieve from cache
let texture_again = assets.get_asset::<Texture>("assets/texture.png");
```

## State Management

### IsotopeState Trait

All game applications implement this trait:

```rust
pub trait IsotopeState: 'static + Send {
    fn init(&mut self, ecs: &Compound, assets: &AssetServer);
    fn update(&mut self, ecs: &Compound, assets: &AssetServer, delta_t: f32, t: f32);
    fn render(&mut self, ecs: &Compound) {}
    fn handle_event(&mut self, event: &WindowEvent) -> bool { false }
}
```

### Lifecycle

1. **Initialization** (`init`): Called once after engine startup
2. **Update Loop** (`update`): Called every frame for game logic
3. **Rendering** (`render`): Called for custom rendering (optional)
4. **Event Handling** (`handle_event`): Called for window events

## Event Handling

Handle user input and window events:

```rust
impl IsotopeState for MyGame {
    fn handle_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { 
                event: KeyEvent { logical_key, state, .. }, 
                .. 
            } => {
                match logical_key {
                    Key::Named(NamedKey::ArrowUp) if *state == ElementState::Pressed => {
                        self.camera_speed += 1.0;
                        true
                    }
                    _ => false
                }
            }
            _ => false
        }
    }
}
```

## Timing and Frame Rate

Control frame timing and physics tick rate:

```rust
pub const ISOTOPE_DEFAULT_TICK_RATE: Duration = Duration::from_micros(50);

// Create custom isotope with different tick rate
let mut isotope = Isotope::new(&event_loop, "My Game", 1920, 1080)?;
isotope.set_tick_rate(Duration::from_millis(16));  // ~60 FPS
```

## Rendering Pipeline

### Deferred Rendering

Isotope uses deferred rendering for efficient multi-light scenes:

1. **Geometry Pass**: Render all geometry to G-buffers
2. **Lighting Pass**: Calculate lighting using G-buffer data
3. **Composition Pass**: Combine results to screen

### Rendering Loop

```
Frame Start
    ↓
Poll Events
    ↓
Update Physics (Boson Thread)
    ↓
Update Game Logic (IsotopeState::update)
    ↓
Geometry Pass
    ↓
Lighting Pass
    ↓
Composition
    ↓
Present to Screen
```

## Complete Example

```rust
use isotope::*;
use winit::event_loop::EventLoop;

#[derive(Default)]
struct GameState {
    time: f32,
}

impl IsotopeState for GameState {
    fn init(&mut self, ecs: &Compound, assets: &AssetServer) {
        // Spawn camera
        ecs.spawn((
            Camera::perspective_3d_default(assets),
            Transform3D::new(
                Vector3::new(0.0, 5.0, 15.0),
                Quaternion::from_axis_angle(Vector3::unit_y(), Deg(0.0)),
            ),
        ));

        // Load and spawn model
        match Model::from_obj("assets/monkey.obj", assets, None) {
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
                5.0,
            ),
        ));
    }

    fn update(&mut self, ecs: &Compound, _assets: &AssetServer, _delta_t: f32, t: f32) {
        self.time = t;

        // Rotate model
        ecs.iter_mut_mol(|_, transform: &mut Transform3D| {
            *transform = Transform3D::new(
                Vector3::new(0.0, 0.0, 0.0),
                Quaternion::from_axis_angle(Vector3::unit_y(), Deg(t * 45.0)),
            );
        });

        // Animate light
        ecs.iter_mut_mol(|_, light: &mut Light| {
            light.pos(|pos| {
                pos[0] = 10.0 * f32::cos(t);
                pos[2] = 10.0 * f32::sin(t);
            });
        });
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let isotope = Isotope::<GameState>::new(
        &event_loop,
        "My Game",
        1920,
        1080,
    ).unwrap();

    event_loop.run_app(&mut isotope.into()).unwrap();
}
```

## Integration Points

### With Compound (ECS)

Access the ECS world to manage entities and components:

```rust
ecs.spawn((component1, component2, ...));
ecs.iter_mol(|entity, component: &ComponentType| { /* ... */ });
ecs.iter_mut_mol(|entity, component: &mut ComponentType| { /* ... */ });
```

### With Boson (Physics)

Physics components integrate with the ECS:

```rust
ecs.spawn((Transform3D::default(), PointMass::new(5.0)));
```

### With Photon (Rendering)

Rendering is automatic, but you can access the renderer:

```rust
// Custom rendering (advanced)
let renderer = isotope.get_renderer();
```

### With GPU Controller

Direct GPU access for advanced graphics:

```rust
let gpu = isotope.get_gpu_controller();
let encoder = gpu.create_command_encoder("Custom");
// Perform GPU operations
gpu.submit(encoder);
```

## Performance Optimization

### Batch Operations

Group similar updates together for cache efficiency:

```rust
// Good: Process all transforms at once
ecs.iter_mut_mol(|_, t: &mut Transform3D| { /* ... */ });

// Less optimal: Process different component types in loop
for _ in 0..1000 {
    ecs.iter_mut_mol(|_, t: &mut Transform3D| { /* ... */ });
    ecs.iter_mut_mol(|_, v: &mut Velocity| { /* ... */ });
}
```

### Use Instancing

For many identical objects, use the instancer:

```rust
let instances = (0..1000).map(|i| {
    Instance::new(
        Vector3::new(i as f32, 0.0, 0.0),
        Quaternion::one(),
        Matrix4::identity(),
    )
}).collect();

ecs.spawn((
    model,
    Instancer::new_serial(Some(0..1000), |instances, _, _| {
        // Update logic
    }),
));
```

### Profile Your Application

Enable logging to identify bottlenecks:

```bash
RUST_LOG=isotope=debug cargo run --release
```

## Window Management

### Configuration

```rust
let isotope = Isotope::new(
    &event_loop,
    "Window Title",   // Window title
    1920,             // Width
    1080,             // Height
)?;
```

### Window Events

Handle window events in your IsotopeState:

```rust
fn handle_event(&mut self, event: &WindowEvent) -> bool {
    match event {
        WindowEvent::Resized(size) => {
            println!("Window resized to {:?}", size);
            true
        }
        WindowEvent::CloseRequested => {
            println!("Close requested");
            true
        }
        _ => false
    }
}
```

## Dependencies

- **photon** - Rendering engine
- **isotope_utils** - Utility functions
- **gpu_controller** - GPU management
- **matter_vault** - Resource utilities
- **compound** - ECS system
- **boson** - Physics engine
- **winit** (0.30.11) - Window and events
- **cgmath** (0.18.0) - 3D mathematics
- **image** (0.25.6) - Image format support
- **smol** (2.0.2) - Async runtime

## Extending Isotope

### Adding New Components

Define custom components and use them in the ECS:

```rust
#[derive(Clone)]
struct MyComponent {
    data: f32,
}

// Spawn with custom component
ecs.spawn((
    MyComponent { data: 42.0 },
    Transform3D::default(),
));

// Query custom component
ecs.iter_mut_mol(|_, comp: &mut MyComponent| {
    comp.data *= 2.0;
});
```

### Adding New Systems

Implement systems as functions operating on component queries:

```rust
fn my_system(ecs: &Compound, delta_t: f32) {
    ecs.iter_mut_duo(|_, a: &mut A, b: &B| {
        // System logic
    });
}

// Call in update
fn update(&mut self, ecs: &Compound, assets: &AssetServer, delta_t: f32, t: f32) {
    my_system(ecs, delta_t);
}
```

## Common Issues

### Models not rendering

1. Check that model file path is correct
2. Verify textures are in correct locations
3. Check that a camera exists in the scene
4. Enable logging: `RUST_LOG=isotope=debug`

### Performance issues

1. Profile with: `cargo build --release`
2. Check frame rate with timestamping
3. Reduce draw calls through instancing
4. Use change detection to avoid redundant updates

### Physics not working

1. Ensure objects have physics components (PointMass, RigidBody)
2. Verify physics thread is running (check logs)
3. Check that objects are spawned before physics updates

## Debugging

Enable detailed logging:

```bash
RUST_LOG=isotope=debug,photon=debug,boson=debug cargo run
```

## Future Enhancements

- [ ] Audio system integration
- [ ] Particle effects system
- [ ] UI framework
- [ ] Animation system
- [ ] Skeletal animation support
- [ ] Advanced physics constraints
- [ ] Network multiplayer support

## API Reference

### Isotope Struct

```rust
impl<T: IsotopeState> Isotope<T> {
    pub fn new(
        event_loop: &EventLoop<()>,
        title: &str,
        width: u32,
        height: u32,
    ) -> Result<Self>
    
    pub fn get_ecs(&self) -> &Arc<Compound>
    pub fn get_asset_server(&self) -> &Arc<AssetServer>
    pub fn get_gpu_controller(&self) -> &Arc<GpuController>
    pub fn get_renderer(&self) -> &Arc<Renderer>
    pub fn set_tick_rate(&mut self, rate: Duration)
}
```

## See Also

- [Main Isotope Documentation](../README.md)
- [Compound ECS](../compound/README.md)
- [Boson Physics](../boson/README.md)
- [Photon Rendering](../photon/README.md)
- [GPU Controller](../gpu_controller/README.md)