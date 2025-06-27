# GPU Controller API Documentation

## Table of Contents

- [Overview](#overview)
- [Core Types](#core-types)
- [GpuController](#gpucontroller)
  - [Constructor](#constructor)
  - [Command Management](#command-management)
  - [Resource Creation](#resource-creation)
  - [Configuration Access](#configuration-access)
- [Error Handling](#error-handling)
- [Examples](#examples)
- [Best Practices](#best-practices)

## Overview

The GPU Controller library provides a high-level interface for managing GPU resources using WGPU. The main entry point is the `GpuController` struct, which encapsulates all GPU operations and resource management.

## Core Types

### GpuController

```rust
pub struct GpuController {
    // Internal fields - not directly accessible
}
```

Thread-safe GPU resource manager that provides simplified access to WGPU functionality.

**Thread Safety**: `GpuController` is designed to be shared across threads using `Arc<GpuController>`.

---

## GpuController

### Constructor

#### `new`

```rust
pub async fn new(
    required_features: Option<Features>,
    required_limits: Option<Limits>,
    surface_configuration: Option<SurfaceConfiguration>,
) -> Result<Arc<Self>>
```

Creates a new GPU controller instance with optional custom configuration.

**Parameters:**

- `required_features: Option<Features>` - Optional WGPU features to enable
  - `None`: Uses default features (recommended for most applications)
  - `Some(features)`: Enables specific WGPU features like texture arrays, compute shaders, etc.

- `required_limits: Option<Limits>` - Optional custom GPU limits
  - `None`: Uses default limits (suitable for most hardware)
  - `Some(limits)`: Custom limits for texture sizes, buffer sizes, binding counts, etc.

- `surface_configuration: Option<SurfaceConfiguration>` - Optional surface configuration
  - `None`: Uses sensible defaults (RGBA8 sRGB, render attachment usage)
  - `Some(config)`: Custom surface format, usage, and presentation settings

**Returns:**
- `Result<Arc<GpuController>>` - Arc-wrapped controller for thread sharing, or error

**Errors:**
- Adapter not found
- Unsupported features or limits
- Device creation failure
- Insufficient system resources

**Example:**

```rust
// Default initialization
let gpu = GpuController::new(None, None, None).await?;

// With custom features
let features = Features::TEXTURE_BINDING_ARRAY | Features::COMPUTE;
let gpu = GpuController::new(Some(features), None, None).await?;

// With custom limits
let mut limits = Limits::default();
limits.max_texture_dimension_2d = 8192;
let gpu = GpuController::new(None, Some(limits), None).await?;
```

---

### Command Management

#### `create_command_encoder`

```rust
pub fn create_command_encoder<S>(&self, label: S) -> CommandEncoder
where S: AsRef<str>
```

Creates a new command encoder for recording GPU commands.

**Parameters:**
- `label: S` - Descriptive label for debugging (appears in graphics tools)

**Returns:**
- `CommandEncoder` - Ready for recording commands

**Usage:**
- Call this method when you need to record GPU commands
- Use the returned encoder to record render passes, compute dispatches, etc.
- Submit the encoder using [`submit`](#submit) when done

**Example:**

```rust
let mut encoder = gpu.create_command_encoder("Frame Render");

// Record commands
{
    let render_pass = encoder.begin_render_pass(&render_pass_desc);
    // ... render commands
}

gpu.submit(encoder);
```

#### `submit`

```rust
pub fn submit(&self, encoder: CommandEncoder)
```

Submits a command encoder to the GPU queue for execution.

**Parameters:**
- `encoder: CommandEncoder` - The encoder containing recorded commands (consumed)

**Behavior:**
- Finalizes the command encoder into a command buffer
- Submits the command buffer to the GPU queue
- Commands execute asynchronously on the GPU
- The encoder is consumed and cannot be used after submission

**Example:**

```rust
let encoder = gpu.create_command_encoder("My Work");
// ... record commands
gpu.submit(encoder); // encoder is consumed here
```

---

### Resource Creation

#### `create_texture`

```rust
pub fn create_texture(&self, texture_descriptor: &TextureDescriptor) -> Texture
```

Creates a new texture resource.

**Parameters:**
- `texture_descriptor: &TextureDescriptor` - Descriptor specifying texture properties

**Returns:**
- `Texture` - The created texture resource

**Common Usage Patterns:**

```rust
// 2D render target texture
let texture_desc = TextureDescriptor {
    label: Some("Render Target"),
    size: Extent3d { width: 1920, height: 1080, depth_or_array_layers: 1 },
    mip_level_count: 1,
    sample_count: 1,
    dimension: TextureDimension::D2,
    format: TextureFormat::Rgba8UnormSrgb,
    usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
    view_formats: &[],
};
let texture = gpu.create_texture(&texture_desc);

// Storage texture for compute
let storage_desc = TextureDescriptor {
    label: Some("Storage Texture"),
    size: Extent3d { width: 512, height: 512, depth_or_array_layers: 1 },
    mip_level_count: 1,
    sample_count: 1,
    dimension: TextureDimension::D2,
    format: TextureFormat::Rgba8Unorm,
    usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
    view_formats: &[],
};
let storage_texture = gpu.create_texture(&storage_desc);
```

#### `create_sampler`

```rust
pub fn create_sampler(&self, sampler_descriptor: &SamplerDescriptor) -> Sampler
```

Creates a new sampler for texture sampling.

**Parameters:**
- `sampler_descriptor: &SamplerDescriptor` - Descriptor specifying sampling behavior

**Returns:**
- `Sampler` - The created sampler resource

**Common Sampler Types:**

```rust
// Linear filtering sampler
let linear_sampler = gpu.create_sampler(&SamplerDescriptor {
    label: Some("Linear Sampler"),
    address_mode_u: AddressMode::Repeat,
    address_mode_v: AddressMode::Repeat,
    address_mode_w: AddressMode::Repeat,
    mag_filter: FilterMode::Linear,
    min_filter: FilterMode::Linear,
    mipmap_filter: FilterMode::Linear,
    ..Default::default()
});

// Nearest neighbor sampler
let nearest_sampler = gpu.create_sampler(&SamplerDescriptor {
    label: Some("Nearest Sampler"),
    address_mode_u: AddressMode::ClampToEdge,
    address_mode_v: AddressMode::ClampToEdge,
    address_mode_w: AddressMode::ClampToEdge,
    mag_filter: FilterMode::Nearest,
    min_filter: FilterMode::Nearest,
    mipmap_filter: FilterMode::Nearest,
    ..Default::default()
});
```

---

### Configuration Access

#### `with_surface_config`

```rust
pub fn with_surface_config<F, R>(&self, callback: F) -> Result<R>
where F: FnOnce(&SurfaceConfiguration) -> R
```

Provides thread-safe read access to the surface configuration.

**Parameters:**
- `callback: F` - Closure that receives the surface configuration and returns a value

**Returns:**
- `Result<R>` - The callback's return value, or an error if lock acquisition fails

**Type Parameters:**
- `F: FnOnce(&SurfaceConfiguration) -> R` - The callback function type
- `R` - The return type of the callback

**Errors:**
- Lock poisoning (rare, indicates panic in another thread)
- Lock contention timeout (very rare)

**Example:**

```rust
// Get surface dimensions
let (width, height) = gpu.with_surface_config(|config| {
    (config.width, config.height)
})?;

// Check surface format
let format = gpu.with_surface_config(|config| config.format)?;

// Get complete configuration copy
let config_copy = gpu.with_surface_config(|config| config.clone())?;
```

---

## Error Handling

The library uses the `anyhow` crate for error handling. All fallible operations return `Result<T>` types.

### Common Error Scenarios

#### Initialization Errors

```rust
match GpuController::new(None, None, None).await {
    Ok(gpu) => {
        // Success - use the GPU controller
    },
    Err(e) => {
        // Handle initialization failure
        eprintln!("GPU initialization failed: {}", e);
        
        // Common causes:
        // - No compatible graphics adapter found
        // - Requested features not supported
        // - Driver issues
        // - Insufficient system resources
    }
}
```

#### Runtime Errors

```rust
// Surface configuration access
match gpu.with_surface_config(|config| config.width) {
    Ok(width) => println!("Surface width: {}", width),
    Err(e) => {
        // Rare - usually indicates lock poisoning
        eprintln!("Failed to access surface config: {}", e);
    }
}
```

### Error Recovery Patterns

```rust
// Retry pattern for initialization
async fn init_gpu_with_fallback() -> anyhow::Result<Arc<GpuController>> {
    // Try with high-end features first
    if let Ok(gpu) = GpuController::new(
        Some(Features::all()), 
        None, 
        None
    ).await {
        return Ok(gpu);
    }
    
    // Fall back to basic features
    GpuController::new(None, None, None).await
}
```

---

## Examples

### Basic Rendering Setup

```rust
use gpu_controller::GpuController;
use wgpu::*;

async fn setup_rendering() -> anyhow::Result<()> {
    // Initialize GPU
    let gpu = GpuController::new(None, None, None).await?;
    
    // Create render target
    let render_target = gpu.create_texture(&TextureDescriptor {
        label: Some("Render Target"),
        size: Extent3d { width: 1920, height: 1080, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    
    // Render a frame
    let mut encoder = gpu.create_command_encoder("Render Frame");
    
    {
        let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Main Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &render_target.create_view(&TextureViewDescriptor::default()),
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        
        // Render commands would go here
    }
    
    gpu.submit(encoder);
    
    Ok(())
}
```

### Compute Shader Example

```rust
use gpu_controller::GpuController;
use wgpu::*;

async fn run_compute() -> anyhow::Result<()> {
    // Initialize with compute features
    let features = Features::COMPUTE;
    let gpu = GpuController::new(Some(features), None, None).await?;
    
    // Create storage texture
    let storage_texture = gpu.create_texture(&TextureDescriptor {
        label: Some("Compute Storage"),
        size: Extent3d { width: 512, height: 512, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    
    // Dispatch compute work
    let mut encoder = gpu.create_command_encoder("Compute Pass");
    
    {
        let compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Main Compute"),
            timestamp_writes: None,
        });
        
        // Compute commands would go here
        // compute_pass.set_pipeline(&compute_pipeline);
        // compute_pass.set_bind_group(0, &bind_group, &[]);
        // compute_pass.dispatch_workgroups(512/8, 512/8, 1);
    }
    
    gpu.submit(encoder);
    
    Ok(())
}
```

### Multi-threaded Usage

```rust
use gpu_controller::GpuController;
use std::sync::Arc;
use std::thread;

async fn multithreaded_usage() -> anyhow::Result<()> {
    let gpu = GpuController::new(None, None, None).await?;
    
    // Clone Arc for sharing across threads
    let gpu_clone = Arc::clone(&gpu);
    
    let handle = thread::spawn(move || {
        // Access surface configuration from thread
        let dimensions = gpu_clone.with_surface_config(|config| {
            (config.width, config.height)
        }).unwrap();
        
        println!("Surface dimensions: {:?}", dimensions);
        
        // Create resources from thread
        let encoder = gpu_clone.create_command_encoder("Background Work");
        // ... record commands
        gpu_clone.submit(encoder);
    });
    
    handle.join().unwrap();
    
    Ok(())
}
```

---

## Best Practices

### Initialization

1. **Start with defaults**: Use `None` for all parameters initially, then customize as needed
2. **Handle initialization failures**: GPU initialization can fail on various systems
3. **Feature detection**: Query adapter capabilities before requesting specific features

```rust
// Good: Start simple
let gpu = GpuController::new(None, None, None).await?;

// Good: Handle failures gracefully
let gpu = match GpuController::new(Some(advanced_features), None, None).await {
    Ok(gpu) => gpu,
    Err(_) => {
        // Fall back to basic features
        GpuController::new(None, None, None).await?
    }
};
```

### Resource Management

1. **Label everything**: Use descriptive labels for debugging
2. **Minimize texture creation**: Reuse textures when possible
3. **Batch command recording**: Group related commands in single encoders

```rust
// Good: Descriptive labels
let encoder = gpu.create_command_encoder("Shadow Map Generation");
let texture = gpu.create_texture(&TextureDescriptor {
    label: Some("Shadow Map 1024x1024"),
    // ...
});

// Good: Batch related work
let mut encoder = gpu.create_command_encoder("Frame Render");
// Record multiple passes
{ /* shadow pass */ }
{ /* main pass */ }
{ /* post-processing pass */ }
gpu.submit(encoder);
```

### Threading

1. **Share the Arc**: Clone `Arc<GpuController>` for thread sharing
2. **Short-lived callbacks**: Keep `with_surface_config` callbacks brief
3. **Command encoding per thread**: Each thread can create its own encoders

```rust
// Good: Clone Arc for sharing
let gpu_clone = Arc::clone(&gpu);
tokio::spawn(async move {
    let encoder = gpu_clone.create_command_encoder("Async Work");
    // ...
    gpu_clone.submit(encoder);
});

// Good: Brief surface config access
let format = gpu.with_surface_config(|config| config.format)?;
```

### Performance

1. **Reuse samplers**: Create samplers once, use many times
2. **Batch submissions**: Submit multiple command buffers together when possible
3. **Profile GPU work**: Use meaningful labels for GPU profiling tools

```rust
// Good: Reuse samplers
let linear_sampler = gpu.create_sampler(&linear_desc);
// Use linear_sampler in multiple bind groups

// Good: Batch submissions (when available in WGPU)
let encoder1 = gpu.create_command_encoder("Pass 1");
let encoder2 = gpu.create_command_encoder("Pass 2");
// Record commands...
gpu.submit(encoder1);
gpu.submit(encoder2);
```
