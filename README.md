# Isotope

A modular, high-performance game engine framework written in Rust, built on WGPU for GPU acceleration and featuring an entity component system architecture. Isotope provides a complete suite of tools for building interactive 3D applications with physics simulation, deferred rendering, and efficient GPU instancing.

## 🎯 Overview

Isotope is a workspace containing multiple specialized libraries that work together to provide a complete game engine solution:

- **Core Engine** (`isotope`) - Main application framework integrating all subsystems
- **GPU Controller** (`gpu_controller`) - Low-level WGPU abstraction layer
- **Renderer** (`photon`) - Deferred rendering engine with lighting
- **Physics Engine** (`boson`) - Physics simulation and rigid body dynamics
- **Entity Component System** (`compound`) - Thread-safe ECS for game object management
- **Asset Management** (`matter_vault`, `isotope_utils`) - Resource loading and utilities
- **Instancing System** - GPU-driven instance rendering for high-performance batch rendering

## 📦 Workspace Structure

```
isotope/
├── boson/              # Physics engine and rigid body dynamics
├── compound/           # Thread-safe Entity Component System (ECS)
├── gpu_controller/     # WGPU abstraction and GPU resource management
├── isotope/            # Core game engine framework
├── isotope_test/       # Example application and testing
├── isotope_utils/      # Utility functions and helpers
├── matter_vault/       # Shared resource management utilities
├── photon/             # Deferred rendering engine with lighting
└── Cargo.toml          # Workspace manifest
```

## 🚀 Getting Started

### Prerequisites

- Rust 1.70+ (uses edition 2024)
- A graphics card supporting WGPU (most modern GPUs)
- Vulkan, Metal, or DX12 backend support

### Building the Project

```bash
# Build all libraries
cargo build

# Build with optimizations
cargo build --release

# Run the test application
cargo run --bin isotope_test --release
```

### Running Tests

```bash
cargo test --workspace
```

## 📚 Library Documentation

### Core Engine (`isotope`)

The main framework that integrates all subsystems. Provides the application loop, window management, event handling, and coordinates between rendering, physics, and ECS systems.

**Key Features:**
- Event-driven application loop
- Window initialization and management
- Physics-ECS integration
- Asset server for resource loading
- Rendering pipeline coordination

**Getting Started:**
```rust
use isotope::*;

struct MyGameState {
    // Your state here
}

impl IsotopeState for MyGameState {
    fn init(&mut self, ecs: &Compound, assets: &AssetServer) {
        // Initialize game
    }

    fn update(&mut self, ecs: &Compound, assets: &AssetServer, delta_t: f32, t: f32) {
        // Update game
    }
}
```

### GPU Controller (`gpu_controller`)

Low-level abstraction over WGPU providing simplified GPU resource management, command submission, and efficient resource caching.

**Key Features:**
- Automatic GPU adapter selection
- Thread-safe resource management
- Bind group layout caching
- Efficient command encoding and submission
- Support for buffers, textures, and samplers

**API Highlights:**
- `GpuController::new()` - Initialize GPU
- `create_command_encoder()` - Create command buffers
- `create_bind_group_layout()` - Create bind groups with caching
- `submit()` - Submit commands to GPU queue

### Renderer (`photon`)

A deferred rendering engine supporting multiple lights, materials, and advanced lighting calculations.

**Key Features:**
- Deferred rendering pipeline
- Multi-light support
- Material system with texturing
- 3D camera with perspective and orthographic projections
- Frustum culling preparation

**Core Components:**
- `Camera3D` - 3D camera management
- `Renderer` - Main rendering pipeline
- `Light` - Lighting information
- Deferred shaders for geometry and lighting passes

### Physics Engine (`boson`)

A physics simulation system supporting point masses, rigid bodies, and static colliders with gravity and collision detection.

**Key Features:**
- Point mass physics
- Rigid body dynamics
- Static colliders
- Gravity support (world and local)
- Multi-threaded physics simulation
- GPU-accelerated particle systems

**Core Types:**
- `PointMass` - Simple mass particles with gravity
- `RigidBody` - Complex bodies with rotation and inertia
- `StaticCollider` - Immovable collision geometry
- `Boson` - Physics world manager

### Entity Component System (`compound`)

A thread-safe, chemistry-inspired ECS implementation with automatic change detection.

**Key Features:**
- Lock-free entity queries (via RwLock per component type)
- Automatic change detection
- Support for multiple component archetypes
- Efficient filtering and exclusion
- Thread-safe component access

**Core Concepts:**
- **Entity** - Unique identifier for game objects
- **Molecule** - Component (analogous to traditional ECS components)
- **Compound** - ECS world managing entities and molecules
- **MoleculeBundle** - Collection of components

**Usage Example:**
```rust
#[derive(Debug)]
struct Position { x: f32, y: f32 }

#[derive(Debug)]
struct Velocity { dx: f32, dy: f32 }

let compound = Compound::new();
let entity = compound.spawn((
    Position { x: 0.0, y: 0.0 },
    Velocity { dx: 1.0, dy: 0.0 },
));

// Iterate over entities with Position and Velocity
compound.iter_mut_duo(|_entity, pos: &mut Position, vel: &Velocity| {
    pos.x += vel.dx;
    pos.y += vel.dy;
});
```

### Asset Management (`matter_vault`)

Shared resource management with thread-safe access patterns and poisoning recovery.

**Key Features:**
- Thread-safe resource wrappers
- Automatic lock poisoning recovery
- Generic resource storage
- Clean callback-based access patterns

### Utilities (`isotope_utils`)

Common utility functions and helpers used across the engine.

## 🎮 Creating Your First Application

1. **Create a new binary crate** in the workspace:
   ```bash
   cargo new --bin my_game
   ```

2. **Add dependencies** to `my_game/Cargo.toml`:
   ```toml
   [dependencies]
   isotope = { path = "../isotope" }
   ```

3. **Implement your game state**:
   ```rust
   use isotope::*;

   struct MyGame;

   impl IsotopeState for MyGame {
       fn init(&mut self, ecs: &Compound, assets: &AssetServer) {
           // Load models, spawn entities, etc.
       }

       fn update(&mut self, ecs: &Compound, assets: &AssetServer, delta_t: f32, t: f32) {
           // Update logic each frame
       }
   }

   fn main() {
       let event_loop = EventLoop::new().unwrap();
       let mut isotope = Isotope::new(MyGame, &event_loop).unwrap();
       isotope.run(event_loop).unwrap();
   }
   ```

## 🔧 Architecture

### Data Flow

```
Input Events (Winit)
        ↓
    Isotope
        ↓
    ┌───┴────┬──────────┐
    ↓        ↓          ↓
  ECS     Physics    Assets
(Compound) (Boson)  (AssetServer)
    ↓        ↓          ↓
    └────┬───┴──────────┘
         ↓
    Renderer (Photon)
         ↓
    GPU (GpuController)
         ↓
    Display
```

### Threading Model

- **Main Thread**: Event loop, window management, input handling
- **Physics Thread**: Boson runs physics simulation on a separate thread
- **Rendering Thread**: GPU commands submitted from main thread

## 🎨 Rendering Features

- **Deferred Rendering**: Efficient multi-light rendering
- **GPU Instancing**: Render thousands of objects efficiently
- **Material System**: Support for textures and material properties
- **Camera System**: Both perspective and orthographic projections
- **Lighting**: Point lights with customizable parameters

## ⚙️ Performance Optimization

### GPU Instancing

Efficiently render many identical objects:

```rust
let instances = vec![
    Instance::new(pos, rotation, scale),
    // ... more instances
];

let model = Model::from_obj("model.obj", assets, Some(&instances))?;
```

### Deferred Rendering

Efficiently handle multiple lights by deferring lighting calculations to a separate pass.

### Thread-Safe ECS

Component data is partitioned by type, allowing multiple systems to access different components simultaneously without blocking.

## 📖 Example: Full Application

See `isotope_test/src/main.rs` for a complete example featuring:
- Model loading from OBJ files
- GPU instancing
- Lighting
- Camera control
- Physics integration

## 🐛 Debugging

Enable debug logging:

```bash
RUST_LOG=debug cargo run --bin isotope_test
```

Optional deadlock detection in tests:

```bash
cargo test --features deadlock_detection
```

## 📝 Code Organization

Each library follows a consistent structure:

```
library/
├── Cargo.toml
└── src/
    ├── lib.rs           # Public API
    ├── module1.rs       # Implementation modules
    └── module2/
        └── submodule.rs
```

## 🤝 Contributing

When adding new features:

1. **Follow existing patterns** - Maintain consistency with current code style
2. **Thread-safe design** - Ensure all shared state uses appropriate synchronization
3. **Document APIs** - Add documentation comments to public items
4. **Test thoroughly** - Add tests for new functionality

## ⚖️ License

[Add your license information here]

## 📞 Support

For issues or questions, refer to the individual library documentation or check the example application in `isotope_test`.

## 🗺️ Roadmap

Potential future enhancements:

- [ ] Audio system
- [ ] Advanced particle effects
- [ ] Networking support
- [ ] Animation system
- [ ] UI framework
- [ ] Terrain rendering
- [ ] Advanced physics constraints
- [ ] Scene serialization