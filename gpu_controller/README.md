# GPU Controller Library

A high-level Rust library that provides a simplified interface for managing GPU resources using the WGPU graphics API. This library abstracts common GPU operations and provides efficient resource management for graphics applications.

## Overview

The GPU Controller library is designed to streamline GPU resource management by providing:

- **Simplified GPU initialization** with sensible defaults
- **Automatic resource management** with interior mutability patterns
- **Efficient bind group layout caching** to reduce redundant GPU object creation
- **Thread-safe access** to GPU resources using Arc and RwLock
- **Flexible configuration** for different use cases

## Architecture

The library consists of three main components:

### Core Components

1. **GpuController** (`src/lib.rs`) - The main controller that manages all GPU resources
2. **LayoutsManager** (`src/layouts.rs`) - Caches and manages WGPU bind group layouts
3. **Defaults** (`src/defaults.rs`) - Provides sensible default configurations

## Features

- **Async GPU initialization** with proper error handling
- **Command encoder creation and submission** with simplified API
- **Texture and sampler creation** with direct device access
- **Surface configuration management** with thread-safe access
- **Comprehensive error handling** using the `anyhow` crate
- **Flexible feature and limits configuration** for different GPU capabilities

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
gpu_controller = { path = "path/to/gpu_controller" }
```

## Dependencies

- **wgpu 25.0.2** - Core graphics API abstraction
- **anyhow 1.0.98** - Error handling
- **log 0.4.27** - Logging support

### Development Dependencies

- **smol 2.0.2** - Async runtime for testing

## Quick Start

```rust
use gpu_controller::GpuController;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the GPU controller with default settings
    let gpu_controller = GpuController::new(None, None, None).await?;
    
    // Create a command encoder
    let mut encoder = gpu_controller.create_command_encoder("My Command Encoder");
    
    // Submit commands to the GPU
    gpu_controller.submit(encoder);
    
    Ok(())
}
```

## API Reference

### GpuController

The main struct that manages all GPU resources.

#### Initialization

```rust
pub async fn new(
    required_features: Option<Features>,
    required_limits: Option<Limits>,
    surface_configuration: Option<SurfaceConfiguration>,
) -> Result<Arc<Self>>
```

Creates a new GPU controller instance with optional custom features, limits, and surface configuration.

**Parameters:**
- `required_features`: Optional WGPU features to enable
- `required_limits`: Optional custom GPU limits
- `surface_configuration`: Optional custom surface configuration

**Returns:** `Result<Arc<GpuController>>` - An Arc-wrapped GPU controller or an error

#### Core Methods

##### Command Encoding

```rust
pub fn create_command_encoder<S>(&self, label: S) -> CommandEncoder
where S: AsRef<str>
```

Creates a new command encoder with the specified label.

```rust
pub fn submit(&self, encoder: CommandEncoder)
```

Submits a command encoder to the GPU queue for execution.

##### Resource Creation

```rust
pub fn create_texture(&self, texture_descriptor: &TextureDescriptor) -> Texture
```

Creates a new texture using the provided descriptor.

```rust
pub fn create_sampler(&self, sampler_descriptor: &SamplerDescriptor) -> Sampler
```

Creates a new sampler using the provided descriptor.

##### Configuration Access

```rust
pub fn with_surface_config<F, R>(&self, callback: F) -> Result<R>
where F: FnOnce(&SurfaceConfiguration) -> R
```

Provides thread-safe read access to the surface configuration through a callback.

### LayoutsManager

Internal component that manages bind group layout caching.

#### Key Features

- **Automatic caching** - Layouts are cached by label to avoid redundant creation
- **Label validation** - Ensures all layouts have valid labels for caching
- **Efficient retrieval** - Fast HashMap-based layout lookup

### Default Configuration

The library provides sensible defaults for surface configuration:

- **Usage:** Render attachment
- **Format:** RGBA8 Unorm sRGB
- **Present Mode:** Auto No VSync
- **Alpha Mode:** Auto
- **Frame Latency:** 2 frames

## Usage Examples

### Basic GPU Initialization

```rust
use gpu_controller::GpuController;
use wgpu::{Features, Limits};

async fn initialize_gpu() -> anyhow::Result<()> {
    // Initialize with custom features
    let features = Features::TEXTURE_BINDING_ARRAY;
    let gpu = GpuController::new(Some(features), None, None).await?;
    
    println!("GPU initialized successfully!");
    Ok(())
}
```

### Texture Creation

```rust
use wgpu::{TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, Extent3d};

fn create_render_texture(gpu: &GpuController) -> wgpu::Texture {
    let texture_desc = TextureDescriptor {
        label: Some("Render Texture"),
        size: Extent3d {
            width: 1920,
            height: 1080,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    };
    
    gpu.create_texture(&texture_desc)
}
```

### Command Submission

```rust
async fn render_frame(gpu: &GpuController) -> anyhow::Result<()> {
    // Create command encoder
    let mut encoder = gpu.create_command_encoder("Frame Render");
    
    // Record rendering commands
    {
        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[/* your attachments */],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        // Add rendering commands here
    }
    
    // Submit to GPU
    gpu.submit(encoder);
    
    Ok(())
}
```

### Surface Configuration Access

```rust
fn get_surface_dimensions(gpu: &GpuController) -> anyhow::Result<(u32, u32)> {
    gpu.with_surface_config(|config| {
        (config.width, config.height)
    })
}
```

## Error Handling

The library uses the `anyhow` crate for comprehensive error handling. All fallible operations return `Result` types:

```rust
match GpuController::new(None, None, None).await {
    Ok(gpu) => {
        // Use the GPU controller
        println!("GPU initialization successful");
    },
    Err(e) => {
        eprintln!("Failed to initialize GPU: {}", e);
        return Err(e);
    }
}
```

## Testing

Run the test suite:

```bash
cargo test
```

The library includes basic initialization tests to ensure the GPU controller can be created successfully.

## Thread Safety

The GPU controller is designed to be thread-safe:

- Uses `Arc<Self>` for shared ownership across threads
- Uses `RwLock` for interior mutability of surface configuration
- WGPU resources are inherently thread-safe where applicable

## Performance Considerations

- **Layout Caching**: Bind group layouts are cached to avoid redundant GPU object creation
- **Arc Usage**: Shared ownership minimizes cloning overhead
- **Command Encoder Reuse**: Create encoders only when needed and submit immediately

## Logging

The library uses the `log` crate for debugging and information output. Initialize a logger in your application to see GPU controller messages:

```rust
env_logger::init();
```

## Contributing

When contributing to this library:

1. Ensure all public methods are documented
2. Add tests for new functionality
3. Follow Rust naming conventions
4. Use the `anyhow` crate for error handling
5. Add appropriate logging statements

## License

[Add your license information here]

## Changelog

### Version 0.1.0
- Initial release
- Basic GPU controller functionality
- Layout management system
- Default configuration support
- Thread-safe surface configuration access