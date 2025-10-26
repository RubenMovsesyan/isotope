# Matter Vault - Thread-Safe Shared Resource Container

A utility library providing thread-safe, lock-aware data container patterns for the Isotope engine. Matter Vault simplifies safe data sharing across threads with automatic poisoned lock recovery.

## Overview

Matter Vault provides a single, well-designed abstraction for safely sharing mutable data between threads. It combines Rust's type safety with runtime lock management to create a reliable foundation for concurrent data access.

**Key Features:**
- Thread-safe wrappers around shared data
- Automatic poisoned lock recovery
- Generic resource storage
- Clean callback-based access patterns
- Zero-copy Arc-based sharing

## Core Concept: SharedMatter<T>

The `SharedMatter<T>` type is a thread-safe wrapper around any type `T`:

```rust
pub struct SharedMatter<T>(Arc<RwLock<T>>);

unsafe impl<T> Send for SharedMatter<T> {}
unsafe impl<T> Sync for SharedMatter<T> {}
```

It's safe to send and sync across thread boundaries, providing thread-safe access through callback patterns.

## Architecture

### Design Pattern

```
Application Thread          Physics Thread
        │                        │
        ├──→ SharedMatter<T> ←──┤
        │   (Arc<RwLock<T>>)    │
        │                        │
    read/write              read/write
    (callbacks)             (callbacks)
        │                        │
        └────────────────────────┘
             Thread-safe access
```

### Lock Management

Matter Vault automatically handles RwLock mechanics:

- **Write callbacks** acquire exclusive write locks
- **Read callbacks** acquire shared read locks
- **Poisoned locks** are automatically recovered
- **No manual lock management** required

## Usage

### Creating Shared Data

```rust
use matter_vault::SharedMatter;

// Create shared data
let shared_value = SharedMatter::new(42);

// Clone for multiple threads
let shared_clone = shared_value.clone();
```

### Reading Data

Use callback-based read access:

```rust
shared_value.read(|value| {
    println!("Value: {}", value);
    // Return any result
    *value * 2
});
```

**Thread-safe characteristics:**
- Multiple threads can read simultaneously
- Readers don't block other readers
- Non-blocking if no writer holds the lock

### Writing Data

Use callback-based write access:

```rust
shared_value.write(|value| {
    *value += 10;
    println!("New value: {}", value);
});
```

**Thread-safe characteristics:**
- Exclusive write access (one writer at a time)
- Writers block readers and other writers
- Safe modification of shared data

### Complete Example

```rust
use matter_vault::SharedMatter;
use std::thread;

fn main() {
    // Create shared data
    let counter = SharedMatter::new(0);

    // Clone for thread 1
    let counter1 = counter.clone();
    let thread1 = thread::spawn(move || {
        for _ in 0..100 {
            counter1.write(|count| {
                *count += 1;
            });
        }
    });

    // Clone for thread 2
    let counter2 = counter.clone();
    let thread2 = thread::spawn(move || {
        for _ in 0..100 {
            counter2.write(|count| {
                *count += 1;
            });
        }
    });

    // Wait for threads
    thread1.join().unwrap();
    thread2.join().unwrap();

    // Read final value
    counter.read(|count| {
        println!("Final count: {}", count);  // Should be 200
    });
}
```

## Lock Poisoning Recovery

When a thread panics while holding a lock, RwLock becomes "poisoned". Matter Vault automatically recovers:

```rust
pub fn read<F, R>(&self, callback: F) -> R
where
    F: FnOnce(&T) -> R,
{
    let matter = match self.0.read() {
        Ok(matter) => matter,
        Err(poisoned) => {
            warn!("Lock was poisoned, recovering...");
            poisoned.into_inner()  // Recover from poisoned lock
        }
    };
    callback(&matter)
}

pub fn write<F, R>(&self, callback: F) -> R
where
    F: FnOnce(&mut T) -> R,
{
    let mut matter = match self.0.write() {
        Ok(matter) => matter,
        Err(poisoned) => {
            warn!("Lock was poisoned, recovering...");
            poisoned.into_inner()  // Recover from poisoned lock
        }
    };
    callback(&mut matter)
}
```

**Benefits:**
- Panics don't permanently deadlock your application
- Automatic recovery with warning logs
- Graceful error handling

### Poisoning Example

```rust
use matter_vault::SharedMatter;
use std::thread;

let shared = SharedMatter::new(vec![1, 2, 3]);
let shared_clone = shared.clone();

// Spawn thread that panics
let thread = thread::spawn(move || {
    shared_clone.write(|data| {
        panic!("Oops!");  // Panics, locks become poisoned
    });
});

// Thread panics (expected)
let _ = thread.join();

// Matter Vault automatically recovers
shared.read(|data| {
    println!("Data still accessible: {:?}", data);
});
```

## Use Cases in Isotope

### 1. Asset Caching

```rust
use matter_vault::SharedMatter;
use std::collections::HashMap;
use std::sync::Arc;

pub struct AssetCache {
    cache: SharedMatter<HashMap<String, Arc<dyn Any>>>,
}

impl AssetCache {
    pub fn new() -> Self {
        Self {
            cache: SharedMatter::new(HashMap::new()),
        }
    }

    pub fn store<T: 'static>(&self, key: String, asset: Arc<T>) {
        self.cache.write(|cache| {
            cache.insert(key, asset as Arc<dyn Any>);
        });
    }

    pub fn retrieve<T: 'static>(&self, key: &str) -> Option<Arc<T>> {
        self.cache.read(|cache| {
            cache.get(key).and_then(|asset| {
                asset.clone().downcast::<T>().ok()
            })
        })
    }
}
```

### 2. Configuration State

```rust
use matter_vault::SharedMatter;

pub struct GameConfig {
    volume: f32,
    resolution: (u32, u32),
    fullscreen: bool,
}

let config = SharedMatter::new(GameConfig {
    volume: 0.8,
    resolution: (1920, 1080),
    fullscreen: false,
});

// Read config from multiple threads
let config_clone = config.clone();
std::thread::spawn(move || {
    config_clone.read(|cfg| {
        println!("Volume: {}", cfg.volume);
    });
});

// Update config
config.write(|cfg| {
    cfg.volume = 1.0;
});
```

### 3. Logging State

```rust
use matter_vault::SharedMatter;

pub struct LogBuffer {
    messages: Vec<String>,
}

let log = SharedMatter::new(LogBuffer {
    messages: Vec::new(),
});

// Log from multiple threads
let log_clone = log.clone();
std::thread::spawn(move || {
    log_clone.write(|buffer| {
        buffer.messages.push("Thread 1 message".to_string());
    });
});

// Read logs
log.read(|buffer| {
    for msg in &buffer.messages {
        println!("{}", msg);
    }
});
```

### 4. Physics State

```rust
use matter_vault::SharedMatter;
use cgmath::Vector3;

pub struct PhysicsState {
    gravity: Vector3<f32>,
    friction: f32,
    time_scale: f32,
}

let physics = SharedMatter::new(PhysicsState {
    gravity: Vector3::new(0.0, -9.81, 0.0),
    friction: 0.1,
    time_scale: 1.0,
});

// Read in physics calculations
let physics_clone = physics.clone();
std::thread::spawn(move || {
    physics_clone.read(|state| {
        println!("Gravity: {:?}", state.gravity);
    });
});

// Adjust at runtime
physics.write(|state| {
    state.time_scale = 0.5;  // Slow motion
});
```

## Performance Considerations

### Lock Contention

Matter Vault uses `parking_lot::RwLock` (via standard `RwLock`) for efficiency:

- **Read locks**: Multiple threads can hold simultaneously
- **Write locks**: Exclusive access (only one writer)
- **No writer starvation**: Write locks have priority

### Usage Tips

1. **Keep callbacks short**
   ```rust
   // ✅ Good: Minimal work under lock
   shared.write(|data| {
       data.value = 42;
   });

   // ❌ Avoid: Heavy computation under lock
   shared.write(|data| {
       for i in 0..1_000_000 {
           data.value += expensive_calculation(i);
       }
   });
   ```

2. **Clone-on-read for long operations**
   ```rust
   // ✅ Better: Clone data, release lock early
   let copy = shared.read(|data| data.clone());
   expensive_operation(&copy);

   // ❌ Avoid: Lock held during expensive operation
   shared.read(|data| {
       expensive_operation(data);
   });
   ```

3. **Batch writes**
   ```rust
   // ✅ Good: Single write transaction
   shared.write(|data| {
       data.field1 = value1;
       data.field2 = value2;
       data.field3 = value3;
   });

   // ❌ Avoid: Multiple lock acquisitions
   shared.write(|data| { data.field1 = value1; });
   shared.write(|data| { data.field2 = value2; });
   shared.write(|data| { data.field3 = value3; });
   ```

## Thread Safety Guarantees

### Safe Operations

✅ **Multiple concurrent readers**
```rust
thread1.join(|| shared.read(|data| { /* read */ }));
thread2.join(|| shared.read(|data| { /* read */ }));
// Both threads execute concurrently
```

✅ **One writer at a time**
```rust
thread1.join(|| shared.write(|data| { /* write */ }));
thread2.join(|| shared.write(|data| { /* write */ }));
// Thread 2 waits for thread 1 to complete
```

✅ **Automatic poisoning recovery**
```rust
thread1.join(|| {
    shared.write(|data| {
        panic!("Error!");  // Lock poisoned
    });
});

// No deadlock - automatic recovery
shared.read(|data| { /* continues */ });
```

### Unsafe Operations

You should NOT directly access the inner RwLock:

```rust
// ❌ Don't do this
unsafe { (*shared.0.get_mut()).force_unlock() }

// ✓ Use the provided API instead
shared.read(|data| { /* safe access */ });
```

## API Reference

### SharedMatter<T>

```rust
impl<T> SharedMatter<T> {
    /// Create a new shared value
    pub fn new(t: T) -> Self

    /// Read with immutable access
    pub fn read<F, R>(&self, callback: F) -> R
    where
        F: FnOnce(&T) -> R

    /// Write with mutable access
    pub fn write<F, R>(&self, callback: F) -> R
    where
        F: FnOnce(&mut T) -> R
}

impl<T> Clone for SharedMatter<T> {
    fn clone(&self) -> Self
}
```

## Generic Usage

SharedMatter works with any type, providing type-safe shared access:

```rust
use matter_vault::SharedMatter;

// Shared primitive
let num = SharedMatter::new(42);

// Shared struct
struct MyData {
    name: String,
    value: i32,
}
let data = SharedMatter::new(MyData {
    name: "test".to_string(),
    value: 100,
});

// Shared collection
let vec = SharedMatter::new(vec![1, 2, 3]);

// Shared enum
enum State {
    Idle,
    Running,
    Complete,
}
let state = SharedMatter::new(State::Idle);
```

## Integration with Isotope

Matter Vault is used throughout Isotope for resource management:

### Asset Server

```rust
pub struct AssetServer {
    cache: SharedMatter<HashMap<String, Arc<dyn Any>>>,
}
```

### Configuration

```rust
pub struct EngineConfig {
    settings: SharedMatter<GameSettings>,
}
```

### State Management

```rust
pub struct GameState {
    data: SharedMatter<StateData>,
}
```

## Dependencies

- **parking_lot** - For efficient lock implementations
- **log** - For warning messages on poisoned locks

## Limitations & Future Work

### Current Limitations

- Single-level locking (no nested locks supported)
- Callback-based API prevents traditional mutable references
- No built-in upgrade from read to write lock

### Planned Features

- [ ] Multi-level nested locking support
- [ ] Lock timeout mechanisms
- [ ] Reader-writer lock statistics
- [ ] Deadlock detection in debug builds
- [ ] Custom lock implementations

## Best Practices

### 1. Keep Data Structures Simple

```rust
// ✅ Good: Simple, focused data
struct CameraState {
    position: Vector3<f32>,
    target: Vector3<f32>,
}

// ❌ Avoid: Too many concerns
struct EverythingData {
    camera: CameraState,
    physics: PhysicsState,
    rendering: RenderState,
    audio: AudioState,
}
```

### 2. Use Appropriate Granularity

```rust
// ✅ Better: Separate locks for independent data
let camera = SharedMatter::new(CameraState::default());
let physics = SharedMatter::new(PhysicsState::default());

// ❌ Avoid: Single lock for everything
let everything = SharedMatter::new(EverythingData::default());
```

### 3. Document Synchronization Requirements

```rust
/// This function expects the camera state to remain stable
/// during the call. Do not modify from other threads.
fn render_with_camera(camera: &CameraState) {
    // Rendering logic
}

// Usage:
camera.read(|cam| {
    render_with_camera(cam);
});
```

### 4. Handle Panics Gracefully

```rust
match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
    shared.write(|data| {
        risky_operation(data);
    });
})) {
    Ok(_) => println!("Operation succeeded"),
    Err(_) => println!("Operation panicked, but lock recovered"),
}
```

## Common Patterns

### Initialization Pattern

```rust
let data = SharedMatter::new(ExpensiveData::new());

// Share with multiple threads
let clone1 = data.clone();
let clone2 = data.clone();

std::thread::spawn(move || {
    clone1.read(|d| { /* use d */ });
});

std::thread::spawn(move || {
    clone2.write(|d| { /* modify d */ });
});
```

### Lazy Initialization Pattern

```rust
pub struct LazyResource {
    data: SharedMatter<Option<Resource>>,
}

impl LazyResource {
    pub fn get_or_init<F: FnOnce() -> Resource>(&self, init: F) {
        self.data.write(|resource| {
            if resource.is_none() {
                *resource = Some(init());
            }
        });
    }
}
```

### Statistics Gathering Pattern

```rust
pub struct Statistics {
    stats: SharedMatter<StatsData>,
}

impl Statistics {
    pub fn record_event(&self, event: String) {
        self.stats.write(|stats| {
            stats.events.push(event);
        });
    }

    pub fn get_summary(&self) -> StatsSummary {
        self.stats.read(|stats| {
            stats.summarize()
        })
    }
}
```

## Debugging

Enable logging for lock poisoning events:

```bash
RUST_LOG=matter_vault=warn cargo run
```

Will output:
```
[WARN] Lock was poisoned, recovering...
```

## See Also

- [Main Isotope Documentation](../README.md)
- [Compound ECS](../compound/README.md)
- [GPU Controller](../gpu_controller/README.md)