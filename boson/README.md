# Boson - Physics Simulation Engine

Boson is a physics simulation library for the Isotope engine, providing rigid body dynamics, point mass simulations, collision detection frameworks, and GPU-accelerated particle systems. It is designed to run on a separate thread for non-blocking physics updates.

## Overview

Boson implements core physics concepts with a focus on performance and integration with GPU-accelerated rendering. The engine supports multiple body types and provides a flexible architecture for extending physics capabilities.

## Features

- **Point Mass Physics**: Simple particle-based physics with gravity support
- **Rigid Body Dynamics**: Full rigid body simulation with rotation and inertia tensors
- **Static Colliders**: Immovable collision geometry for level design
- **Gravity Systems**: Both world-space and local gravity
- **Multi-threaded Simulation**: Physics runs on a dedicated thread with configurable tick rate
- **GPU Integration**: Direct integration with GpuController for GPU-accelerated computations
- **Thread-Safe Objects**: Arc<RwLock<>> wrapped physics bodies for safe concurrent access

## Core Concepts

### BosonBody

The `BosonBody` enum represents different types of physics objects:

```rust
pub enum BosonBody {
    PointMass(PointMass),      // Particle with mass and velocity
    RigidBody(RigidBody),      // Complex body with rotation
    StaticCollider(StaticCollider),  // Immovable collision geometry
}
```

### BosonObject

A thread-safe wrapper around `BosonBody`:

```rust
pub struct BosonObject(Arc<RwLock<BosonBody>>);
```

Provides safe concurrent access to physics bodies from multiple threads.

### Boson World

The main physics engine managing all objects and simulation:

```rust
pub struct Boson {
    objects_count: AtomicU32,
    objects: Arc<RwLock<Vec<BosonObject>>>,
    gpu_controller: Arc<GpuController>,
    tickrate: Duration,
    boson_thread: (Arc<RwLock<bool>>, JoinHandle<()>),
}
```

## Architecture

### Threading Model

```
Main Thread
    ↓
    ├─→ BosonObject access (read/write)
    │       ↑
    │       │
Physics Thread ←─ Boson Physics Loop
    │
    └─→ Updates bodies based on forces/gravity
```

Boson runs physics simulation on a dedicated thread:
- Default tick rate: 50 microseconds (20,000 Hz)
- Non-blocking updates to physics bodies
- Thread-safe access from main application thread

### Gravity System

Boson supports flexible gravity configurations:

```rust
pub enum Gravity {
    World(Vector3<f32>),    // Global acceleration (e.g., -9.81 Y)
    Local(Vector3<f32>),    // Per-object acceleration
}
```

## Body Types

### PointMass

A simple particle with mass and velocity, affected by forces and gravity.

**Use cases:**
- Particle effects
- Simple projectiles
- Rain/snow systems
- Debris simulations

**Properties:**
- `mass`: Mass in kilograms
- `velocity`: Linear velocity vector
- `position`: Current world position
- `forces`: Accumulated forces

**Example:**
```rust
let point_mass = PointMass::new(1.5);  // 1.5 kg particle
```

### RigidBody

A complex body with rotation, inertia, and angular velocity.

**Use cases:**
- Characters
- Props and movable objects
- Vehicles
- Interactive elements

**Properties:**
- `mass`: Total mass
- `velocity`: Linear velocity
- `angular_velocity`: Rotation speed
- `inertia_tensor`: Resistance to rotation
- `forces`: Applied forces
- `torques`: Applied rotational forces

### StaticCollider

Immovable collision geometry, typically used for level geometry.

**Use cases:**
- Walls and floors
- Terrain
- Buildings
- Level architecture

**Properties:**
- Fixed position and orientation
- No mass or forces
- Collision surfaces defined by shape

## Usage Examples

### Initializing Boson

```rust
use boson::Boson;
use gpu_controller::GpuController;
use std::sync::Arc;

// Initialize GPU first
let gpu = Arc::new(GpuController::new(None, None, None).await?);

// Create physics world
let mut boson = Boson::new(gpu);
```

### Adding Physics Objects

```rust
use boson::{BosonObject, PointMass, BosonBody};
use cgmath::Vector3;

// Create a point mass object
let point_mass = BosonObject::new(BosonBody::PointMass(PointMass::new(2.0)));

// Register with physics world
let object_id = boson.add_object(&point_mass);
```

### Modifying Objects

```rust
// Modify physics body through safe callback
point_mass.modify_body(|body| {
    if let BosonBody::PointMass(ref mut pm) = body {
        // Apply forces, change properties, etc.
    }
});

// Read physics body
point_mass.read_body(|body| {
    if let BosonBody::PointMass(ref pm) = body {
        println!("Position: {:?}", pm.position);
    }
});
```

### Collision Resolution

```rust
// Resolve collisions between two objects
object1.resolve_collisions(&object2, 0.016);  // 16ms timestep
```

## Configuration

### Tick Rate

The physics simulation runs at a configurable tick rate (default: 50 microseconds):

```rust
const DEFAULT_TICKRATE: Duration = Duration::from_micros(50);
```

To adjust:
1. Modify `DEFAULT_TICKRATE` constant
2. Rebuild the library
3. Tick rate affects simulation frequency and stability

### Gravity

Configure gravity in the physics thread:

```rust
let gravity = Gravity::World(Vector3::unit_y() * -9.81);  // Earth-like gravity
```

## Integration with Isotope

Boson integrates with Isotope's ECS through the `BosonCompat` bridge:

```rust
use isotope::{BosonCompat, PointMass, RigidBody};

// Components are available in Isotope
ecs.spawn((
    Transform3D::new(...),
    PointMass::new(5.0),  // Physics component
));
```

## Performance Considerations

### Multi-threading Benefits
- Physics simulation doesn't block rendering
- Smooth frame rates even with heavy physics calculations
- Better CPU utilization on multi-core systems

### Optimization Tips
1. **Reuse BosonObject instances** - Clone the Arc, don't recreate
2. **Batch collision checks** - Check only relevant pairs
3. **Use appropriate body types** - PointMass is simpler than RigidBody
4. **Adjust tick rate** - Higher precision = lower performance

### Memory Efficiency
- Uses Arc<RwLock<>> for minimal allocation overhead
- Physics bodies stored contiguously in vector
- Efficient collision detection data structures

## Extending Boson

### Adding New Body Types

To add a new physics body type:

1. Define a new struct in a module (e.g., `soft_body.rs`)
2. Add to `BosonBody` enum
3. Implement physics calculations in physics thread
4. Update collision detection if needed

### Adding Forces

Custom forces can be applied through `modify_body`:

```rust
object.modify_body(|body| {
    if let BosonBody::PointMass(ref mut pm) = body {
        // Add drag force
        let drag = pm.velocity * -0.5;
        pm.forces += drag;
    }
});
```

### Custom Colliders

Extend `StaticCollider` with new geometry types:

```rust
pub enum ColliderShape {
    Sphere { radius: f32 },
    Box { extents: Vector3<f32> },
    Mesh { vertices: Vec<Vector3<f32>> },
    // Add custom shapes
}
```

## GPU Integration

Boson can leverage GPU acceleration for:

- **Particle System Simulations**: WGSL compute shaders for particle updates
- **Collision Detection**: GPU-accelerated spatial queries
- **Batch Force Calculations**: Parallel force calculations

Access the GPU controller:

```rust
let gpu = &boson.gpu_controller;
let encoder = gpu.create_command_encoder("Physics");
// Submit GPU work
gpu.submit(encoder);
```

## Properties System

Boson includes a flexible properties system for extending physics calculations:

```rust
pub mod properties {
    pub trait Gravitational {
        fn apply_gravity(&mut self, gravity: &Gravity, dt: f64);
    }
}
```

Currently supports:
- **Gravity** - World and local gravitational acceleration

Additional properties can be added by implementing traits.

## Common Issues

### Physics Objects Not Moving

1. **Check thread is running** - Ensure `Boson::new()` completed successfully
2. **Verify forces applied** - Use `read_body()` to check forces
3. **Check tick rate** - If too slow, adjustments take longer
4. **Gravity direction** - Verify gravity vector points in expected direction

### Instability

1. **Reduce tick rate** - Increase frequency for better stability
2. **Check mass values** - Very small or large masses can cause issues
3. **Limit forces** - Clamp force magnitudes
4. **Use fixed timestep** - Consistent dt for stable simulation

### Performance Issues

1. **Profile with perf/flamegraph** - Identify bottlenecks
2. **Reduce collision checks** - Use spatial partitioning
3. **Consider LOD** - Disable physics for distant objects
4. **Batch updates** - Minimize lock contention

## Debugging

Enable detailed logging:

```bash
RUST_LOG=boson=debug cargo run
```

Log output includes:
- Physics thread initialization
- Object additions/removals
- Force applications
- Collision events

## Future Enhancements

- [ ] Soft body physics
- [ ] Fluid dynamics simulation
- [ ] Constraint systems
- [ ] Advanced collision shapes
- [ ] Deterministic playback
- [ ] Physics prediction
- [ ] Ragdoll systems
- [ ] Vehicle dynamics

## Dependencies

- **cgmath** (0.18.0) - 3D mathematics
- **gpu_controller** - GPU resource management
- **parking_lot** (0.12.5) - Efficient synchronization
- **log** (0.4.28) - Logging framework

## API Reference

### Boson

```rust
impl Boson {
    pub fn new(gpu_controller: Arc<GpuController>) -> Self
    pub fn add_object(&mut self, object: &BosonObject) -> u32
}
```

### BosonObject

```rust
impl BosonObject {
    pub fn new(boson_body: BosonBody) -> Self
    pub fn resolve_collisions(&self, other: &BosonObject, timestep: f32)
    pub fn modify_body<F, R>(&self, callback: F) -> R
    pub fn read_body<F, R>(&self, callback: F) -> R
}
```

## Contributing

When contributing to Boson:

1. Follow physics simulation best practices
2. Maintain thread safety with all shared state
3. Add tests for new body types or forces
4. Document changes to physics behavior
5. Profile performance changes

## See Also

- [Isotope Main README](../README.md)
- [GPU Controller](../gpu_controller/README.md)
- [Compound ECS](../compound/README.md)