# Compound

A high-performance, thread-safe Entity Component System (ECS) library for Rust with chemistry-inspired naming conventions.

## Overview

Compound is a modern ECS implementation that provides safe concurrent access to entity-component data through smart use of locks and atomic operations. It uses intuitive chemistry metaphors where components are "molecules" that combine to form "compounds" (entities).

### Key Features

- 🔒 **Thread-Safe by Design** - All operations are thread-safe through RwLock and atomic operations
- ⚡ **High Performance** - Lock-free entity ID generation and efficient component storage
- 🧪 **Chemistry-Inspired API** - Intuitive naming with molecules (components) and compounds (worlds)
- 🔄 **Flexible Iteration** - Multiple query patterns for iterating over entities with specific component combinations
- 📦 **Bundle Support** - Group related components together for easy entity spawning
- 🔍 **Change Detection** - Built-in modified flag system with modified, unmodified, and change-detection variants for all iterators
- 🛡️ **Poison Recovery** - Automatic recovery from poisoned locks with logging

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
compound = "0.1.0"
```

## Quick Start

```rust
use compound::Compound;

// Define your components (molecules)
struct Position {
    x: f32,
    y: f32,
}

struct Velocity {
    dx: f32,
    dy: f32,
}

struct Health {
    current: i32,
    max: i32,
}

fn main() {
    // Create a new ECS world (compound)
    let world = Compound::new();

    // Spawn entities with components
    let player = world.spawn((
        Position { x: 0.0, y: 0.0 },
        Velocity { dx: 1.0, dy: 0.0 },
        Health { current: 100, max: 100 },
    ));

    let enemy = world.spawn((
        Position { x: 10.0, y: 10.0 },
        Health { current: 50, max: 50 },
    ));

    // Query and iterate over entities
    world.iter_duo(|entity, pos: &Position, vel: &Velocity| {
        println!("Entity {} at ({}, {}) moving ({}, {})",
                 entity, pos.x, pos.y, vel.dx, vel.dy);
    });

    // Mutate components
    world.iter_mut_mol(|entity, health: &mut Health| {
        health.current -= 10;
        println!("Entity {} health: {}/{}", entity, health.current, health.max);
    });
}
```

## Core Concepts

### Entities
Entities are unique identifiers (u64) that represent game objects. They are lightweight and serve as keys to access associated components.

### Molecules (Components)
Molecules are the data components that can be attached to entities. Any struct that implements `Send + Sync + 'static` can be used as a molecule.

```rust
struct Transform {
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
}

struct Sprite {
    texture: TextureHandle,
    color: Color,
}
```

### Compound (World)
The Compound is the main ECS container that manages all entities and their molecules. It provides methods for creating entities, adding/removing components, and querying.

### MoleculeBundle
Bundles allow you to group related components together for convenient entity creation:

```rust
// Components are automatically bundled when passed as tuples
let entity = world.spawn((
    Position { x: 0.0, y: 0.0 },
    Velocity { dx: 0.0, dy: 0.0 },
    Sprite { /* ... */ },
));
```

### Change Detection
Compound includes a built-in change detection system that automatically tracks when entities are modified. This enables efficient reactive systems that only process entities when they've actually changed.

The system works through:
- **Modified Flag**: Each entity has an internal `Modified` component that tracks whether it has been changed
- **Modified Iterators**: Special `*_mod` iterator variants that only process entities marked as modified
- **Unmodified Iterators**: Special `*_unmod` iterator variants that provide mutable access without setting the modified flag
- **Automatic Management**: The system automatically sets/clears modified flags during iterations

```rust
// Only process entities that have been modified since last frame
world.iter_mol_mod(|entity, pos: &Position| {
    println!("Entity {} moved to ({}, {})", entity, pos.x, pos.y);
    // This only prints for entities that were actually modified
});

// Modify some entities
world.iter_mut_mol(|entity, pos: &mut Position| {
    pos.x += 1.0; // This sets the modified flag
});

// Now the modified iterator will process these entities
world.iter_mol_mod(|entity, pos: &Position| {
    println!("Modified entity {} at ({}, {})", entity, pos.x, pos.y);
});
```

## Iterator Variants

Compound provides three types of iterator variants for each query pattern, giving you fine-grained control over change detection and performance:

### Standard Iterators
- **Read-only**: `iter_mol`, `iter_duo`, `iter_trio` - Read-only access to components
- **Mutable**: `iter_mut_mol`, `iter_mut_duo`, `iter_mut_trio` - Mutable access that **automatically sets** the modified flag

These are the most common iterators. Mutable variants mark entities as changed, triggering change detection systems.

### Modified-Only Iterators (`*_mod`)
- **Read-only**: `iter_mol_mod`, `iter_duo_mod`, `iter_trio_mod` - Only process entities that have been marked as modified
- **Mutable**: `iter_mut_mol_mod`, `iter_mut_duo_mod`, `iter_mut_trio_mod` - Mutable access to only modified entities, **clears** the modified flag after processing

These iterators enable reactive systems that only run when data actually changes, providing excellent performance for expensive operations.

### Unmodified Iterators (`*_unmod`)
- **Mutable**: `iter_mut_mol_unmod`, `iter_mut_duo_unmod`, `iter_mut_trio_unmod` - Mutable access that **does NOT affect** the modified flag

These iterators are perfect for:
- Internal maintenance operations (cleanup, normalization)
- Systems that shouldn't trigger change detection
- Performance-critical code that needs to avoid flag overhead

### Without Variants
All iterator types also support "without" variants that filter out entities with specific components:
- `iter_without_mol::<Without, Component>`
- `iter_mut_without_duo::<Without, Comp1, Comp2>`
- `iter_mut_without_trio_unmod::<Without, Comp1, Comp2, Comp3>`

### Performance Characteristics

| Iterator Type | Modified Flag | Use Case | Performance |
|--------------|---------------|----------|-------------|
| Standard | Sets on write | General gameplay logic | Fast |
| Modified-only | Reads & clears | Reactive systems | Very fast (sparse) |
| Unmodified | No interaction | Internal maintenance | Fastest |

```rust
// Standard: marks entities as modified
world.iter_mut_mol(|entity, pos: &mut Position| {
    pos.x += 1.0; // This will trigger change detection
});

// Modified-only: processes only changed entities
world.iter_mut_mol_mod(|entity, vel: &mut Velocity| {
    // Only runs for entities modified since last call
    vel.normalize();
});

// Unmodified: invisible to change detection
world.iter_mut_mol_unmod(|entity, pos: &mut Position| {
    // Clean up floating point errors without triggering systems
    pos.x = pos.x.round();
});
```

### When to Use Each Iterator Type

**Standard Iterators** - Use for general gameplay logic:
- Movement systems that update position based on velocity
- Combat systems that modify health/damage
- AI systems that change entity behavior
- Any system where modifications should trigger dependent systems

**Modified-Only Iterators** - Use for expensive reactive systems:
- Graphics systems that only re-render changed objects
- Physics systems that recalculate collisions for moved entities
- Audio systems that update 3D positioned sounds
- Networking systems that only sync changed entities
- Any system that's expensive and should only run when needed

**Unmodified Iterators** - Use for maintenance and optimization:
- Floating-point error correction and normalization
- Memory cleanup and garbage collection
- Debug systems that shouldn't affect gameplay
- Performance monitoring and profiling
- Internal state synchronization between frames

## API Reference

### Entity Management

#### `create_entity() -> Entity`
Creates a new empty entity and returns its ID.

```rust
let entity = world.create_entity();
```

#### `spawn(bundle: impl MoleculeBundle) -> Entity`
Creates a new entity with the provided component bundle.

```rust
let entity = world.spawn((Position::default(), Health::new(100)));
```

#### `add_molecule(entity: Entity, molecule: T)`
Adds a component to an existing entity.

```rust
world.add_molecule(entity, Velocity { dx: 1.0, dy: 0.0 });
```

### Querying

#### Single Component Queries

**Standard Iterators:**
- `iter_mol(f)` - Iterate over all entities with a specific component
- `iter_mut_mol(f)` - Iterate with mutable access to the component
- `iter_without_mol::<W, T>(f)` - Iterate over entities with T but WITHOUT W

**Modified-Only Iterators:**
- `iter_mol_mod(f)` - Iterate over only modified entities with a specific component
- `iter_mut_mol_mod(f)` - Mutable iteration over only modified entities
- `iter_without_mol_mod::<W, T>(f)` - Modified entities with T but without W

**Unmodified Iterators:**
- `iter_mut_mol_unmod(f)` - Mutable iteration without setting the modified flag
- `iter_mut_without_mol_unmod::<W, T>(f)` - Mutable iteration with T but without W, without setting modified flag

```rust
// Read-only iteration
world.iter_mol(|entity, pos: &Position| {
    println!("Entity {} at ({}, {})", entity, pos.x, pos.y);
});

// Only process entities that changed since last check
world.iter_mol_mod(|entity, pos: &Position| {
    println!("Entity {} moved to ({}, {})", entity, pos.x, pos.y);
});

// Mutable iteration (sets modified flag)
world.iter_mut_mol(|entity, vel: &mut Velocity| {
    vel.dx *= 0.99; // Apply friction
});

// Mutable iteration over only already-modified entities
world.iter_mut_mol_mod(|entity, vel: &mut Velocity| {
    // Only process velocities that were already modified
    vel.dx = vel.dx.clamp(-10.0, 10.0);
});

// Mutable iteration without triggering change detection
world.iter_mut_mol_unmod(|entity, vel: &mut Velocity| {
    // Internal cleanup that shouldn't mark entities as "changed"
    vel.normalize_if_too_small();
});
```

#### Two Component Queries

**Standard Iterators:**
- `iter_duo(f)` - Iterate over entities with two specific components
- `iter_mut_duo(f)` - Iterate with mutable access to both components
- `iter_without_duo::<W, T1, T2>(f)` - Iterate over entities with T1, T2 but without W

**Modified-Only Iterators:**
- `iter_duo_mod(f)` - Iterate over only modified entities with two components
- `iter_mut_duo_mod(f)` - Mutable iteration over modified entities with two components
- `iter_without_duo_mod::<W, T1, T2>(f)` - Modified entities with T1, T2 but without W

**Unmodified Iterators:**
- `iter_mut_duo_unmod(f)` - Mutable iteration over two components without setting modified flag
- `iter_mut_without_duo_unmod::<W, T1, T2>(f)` - Mutable iteration with T1, T2 but without W, without setting modified flag

```rust
// Update physics for entities with position and velocity
world.iter_mut_duo(|entity, pos: &mut Position, vel: &Velocity| {
    pos.x += vel.dx;
    pos.y += vel.dy;
});

// Only update physics for entities that have changed
world.iter_duo_mod(|entity, pos: &Position, vel: &Velocity| {
    println!("Entity {} moved to ({}, {})", entity, pos.x, pos.y);
});
```

#### Three Component Queries

**Standard Iterators:**
- `iter_trio(f)` - Iterate over entities with three specific components
- `iter_mut_trio(f)` - Iterate with mutable access to all three components
- `iter_without_trio::<W, T1, T2, T3>(f)` - Iterate over entities with T1, T2, T3 but without W

**Modified-Only Iterators:**
- `iter_trio_mod(f)` - Iterate over only modified entities with three components
- `iter_mut_trio_mod(f)` - Mutable iteration over modified entities with three components
- `iter_without_trio_mod::<W, T1, T2, T3>(f)` - Modified entities with T1, T2, T3 but without W

**Unmodified Iterators:**
- `iter_mut_trio_unmod(f)` - Mutable iteration over three components without setting modified flag
- `iter_mut_without_trio_unmod::<W, T1, T2, T3>(f)` - Mutable iteration with T1, T2, T3 but without W, without setting modified flag

```rust
world.iter_trio(|entity, pos: &Position, vel: &Velocity, health: &Health| {
    println!("Entity {} is a full game object", entity);
});

// Only process complex entities that have been modified
world.iter_trio_mod(|entity, pos: &Position, vel: &Velocity, acc: &Acceleration| {
    println!("Modified physics entity {} needs recalculation", entity);
});
```

## Thread Safety

Compound is designed for concurrent access from multiple threads:

```rust
use std::sync::{Arc, RwLock};
use std::thread;

let world = Arc::new(RwLock::new(Compound::new()));

// Spawn entities from one thread
let world_clone = world.clone();
let spawner = thread::spawn(move || {
    let mut w = world_clone.write().unwrap();
    for i in 0..100 {
        w.spawn((
            Position { x: i as f32, y: 0.0 },
            Health { current: 100, max: 100 },
        ));
    }
});

// Query from another thread
let world_clone = world.clone();
let reader = thread::spawn(move || {
    let w = world_clone.read().unwrap();
    w.iter_mol(|entity, pos: &Position| {
        println!("Entity {} at x={}", entity, pos.x);
    });
});

spawner.join().unwrap();
reader.join().unwrap();
```

### Lock Poisoning Recovery

Compound automatically recovers from poisoned locks (when a thread panics while holding a lock) and logs warnings:

```rust
// Even if a panic occurs during iteration, other threads can continue
world.iter_mut_mol(|entity, data: &mut MyComponent| {
    if data.value == 42 {
        panic!("Don't panic!"); // Lock will be poisoned
    }
});

// This will still work, with a warning logged
world.iter_mol(|entity, data: &MyComponent| {
    println!("Recovered and continuing: {}", data.value);
});
```

## Advanced Usage

### Custom Queries

Build complex queries by combining iteration methods:

```rust
// Find all entities with Position but without Health (e.g., decorative objects)
let mut decorations = Vec::new();
world.iter_mol(|entity, pos: &Position| {
    decorations.push((entity, *pos));
});

world.iter_mol(|entity, _: &Health| {
    decorations.retain(|(e, _)| *e != entity);
});
```

### Component Storage

Compound uses a `HashMap<TypeId, Box<dyn Any>>` internally for type-erased component storage, with each component type stored in its own `MoleculeStorage`. This provides:

- O(1) component type lookup
- Cache-friendly iteration over components of the same type
- Type safety through Rust's type system

## Performance Considerations

1. **Entity Creation**: Uses atomic counters for lock-free ID generation
2. **Component Access**: Read locks allow multiple concurrent readers
3. **Iteration**: Efficient linear iteration over component arrays
4. **Memory**: Components are stored contiguously per type for cache efficiency

## Examples

### Game Loop Example

```rust
fn game_loop(world: &Compound) {
    // Physics update - sets modified flag for moving entities
    world.iter_mut_duo(|_entity, pos: &mut Position, vel: &Velocity| {
        pos.x += vel.dx;
        pos.y += vel.dy;
    });

    // Collision detection - only check entities that moved
    world.iter_duo_mod(|e1, p1: &Position, h1: &Health| {
        world.iter_duo(|e2, p2: &Position, h2: &Health| {
            if e1 != e2 && distance(p1, p2) < COLLISION_RADIUS {
                // Handle collision
            }
        });
    });

    // Render only entities that changed position
    world.iter_duo_mod(|_entity, pos: &Position, sprite: &Sprite| {
        render_sprite(sprite, pos);
    });
}
```

### Change Detection Example

```rust
struct Transform { x: f32, y: f32, dirty_matrix: bool }
struct WorldMatrix([[f32; 4]; 4]);

fn update_transforms(world: &Compound) {
    // Only recalculate world matrices for entities that moved
    world.iter_mut_duo_mod(|_entity, transform: &mut Transform, matrix: &mut WorldMatrix| {
        if transform.dirty_matrix {
            // Expensive matrix calculation only happens for modified entities
            *matrix = calculate_world_matrix(transform);
            transform.dirty_matrix = false;
        }
    });
}

fn movement_system(world: &Compound) {
    // Moving entities will be automatically marked as modified
    world.iter_mut_mol(|_entity, transform: &mut Transform| {
        transform.x += 1.0;
        transform.dirty_matrix = true;
    });
}
```

### System Pattern

```rust
trait System {
    fn update(&self, world: &Compound);
}

struct MovementSystem;
impl System for MovementSystem {
    fn update(&self, world: &Compound) {
        world.iter_mut_duo(|_entity, pos: &mut Position, vel: &Velocity| {
            pos.x += vel.dx;
            pos.y += vel.dy;
        });
    }
}

struct HealthSystem;
impl System for HealthSystem {
    fn update(&self, world: &Compound) {
        world.iter_mut_mol(|entity, health: &mut Health| {
            if health.current <= 0 {
                println!("Entity {} has died", entity);
                // Mark for removal
            }
        });
    }
}

struct RenderSystem;
impl System for RenderSystem {
    fn update(&self, world: &Compound) {
        // Only render entities that have moved since last frame
        world.iter_duo_mod(|_entity, pos: &Position, sprite: &Sprite| {
            render_sprite_at_position(sprite, pos);
        });
    }
}

struct CleanupSystem;
impl System for CleanupSystem {
    fn update(&self, world: &Compound) {
        // Perform internal maintenance without triggering change detection
        world.iter_mut_mol_unmod(|_entity, vel: &mut Velocity| {
            // Clean up floating point errors without marking as "changed"
            vel.dx = if vel.dx.abs() < 0.001 { 0.0 } else { vel.dx };
            vel.dy = if vel.dy.abs() < 0.001 { 0.0 } else { vel.dy };
        });
    }
}

struct PhysicsSystem;
impl System for PhysicsSystem {
    fn update(&self, world: &Compound) {
        // Only recalculate physics for entities that changed
        world.iter_mut_trio_mod(|_entity, pos: &mut Position, vel: &mut Velocity, acc: &Acceleration| {
            vel.dx += acc.dx;
            vel.dy += acc.dy;
            pos.x += vel.dx;
            pos.y += vel.dy;
        });
    }
}
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/isotope.git
cd isotope/compound

# Run tests
cargo test

# Run benchmarks
cargo bench

# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy -- -D warnings
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Inspired by various ECS implementations in the Rust ecosystem
- Chemistry naming convention inspired by the molecular composition metaphor
- Built with safety and performance as primary goals
