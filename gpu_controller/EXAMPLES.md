# GPU Controller Examples

This document provides comprehensive examples showing how to use the GPU Controller library in various scenarios, from basic initialization to complex rendering pipelines.

## Table of Contents

- [Basic Initialization](#basic-initialization)
- [Texture Creation](#texture-creation)
- [Command Recording and Submission](#command-recording-and-submission)
- [Render Passes](#render-passes)
- [Compute Shaders](#compute-shaders)
- [Multi-threaded Usage](#multi-threaded-usage)
- [Error Handling](#error-handling)
- [Performance Optimization](#performance-optimization)
- [Real-world Scenarios](#real-world-scenarios)

---

## Basic Initialization

### Default Initialization

The simplest way to initialize the GPU controller:

```rust
use gpu_controller::GpuController;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize with all default settings
    let gpu = GpuController::new(None, None, None).await?;
    
    println!("GPU Controller initialized successfully!");
    
    Ok(())
}
```

### Custom Features

Initialize with specific WGPU features:

```rust
use gpu_controller::GpuController;
use wgpu::Features;

async fn init_with_features() -> anyhow::Result<Arc<GpuController>> {
    let features = Features::TEXTURE_BINDING_ARRAY 
        | Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
        | Features::COMPUTE;
    
    let gpu = GpuController::new(Some(features), None, None).await?;
    
    println!("GPU initialized with advanced features");
    Ok(gpu)
}
```

### Custom Limits

Initialize with custom GPU limits:

```rust
use gpu_controller::GpuController;
use wgpu::Limits;

async fn init_with_limits() -> anyhow::Result<Arc<GpuController>> {
    let mut limits = Limits::default();
    limits.max_texture_dimension_2d = 8192;
    limits.max_buffer_size = 256 * 1024 * 1024; // 256MB
    limits.max_bind_groups = 8;
    
    let gpu = GpuController::new(None, Some(limits), None).await?;
    
    println!("GPU initialized with custom limits");
    Ok(gpu)
}
```

### Custom Surface Configuration

Initialize with a custom surface configuration:

```rust
use gpu_controller::GpuController;
use wgpu::{SurfaceConfiguration, TextureUsages, TextureFormat, PresentMode, CompositeAlphaMode};

async fn init_with_surface_config() -> anyhow::Result<Arc<GpuController>> {
    let surface_config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: TextureFormat::Bgra8UnormSrgb,
        width: 1920,
        height: 1080,
        present_mode: PresentMode::Fifo, // VSync enabled
        alpha_mode: CompositeAlphaMode::Opaque,
        view_formats: vec![],
        desired_maximum_frame_latency: 3,
    };
    
    let gpu = GpuController::new(None, None, Some(surface_config)).await?;
    
    println!("GPU initialized with custom surface configuration");
    Ok(gpu)
}
```

---

## Texture Creation

### Render Target Texture

Create a texture for rendering:

```rust
use wgpu::{TextureDescriptor, Extent3d, TextureDimension, TextureFormat, TextureUsages};

fn create_render_target(gpu: &GpuController, width: u32, height: u32) -> wgpu::Texture {
    let texture_desc = TextureDescriptor {
        label: Some("Main Render Target"),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };
    
    gpu.create_texture(&texture_desc)
}
```

### Depth Buffer

Create a depth texture for 3D rendering:

```rust
fn create_depth_texture(gpu: &GpuController, width: u32, height: u32) -> wgpu::Texture {
    let depth_desc = TextureDescriptor {
        label: Some("Depth Buffer"),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Depth32Float,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };
    
    gpu.create_texture(&depth_desc)
}
```

### Texture Array

Create a texture array for multiple images:

```rust
fn create_texture_array(gpu: &GpuController, width: u32, height: u32, layers: u32) -> wgpu::Texture {
    let array_desc = TextureDescriptor {
        label: Some("Texture Array"),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: layers,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    };
    
    gpu.create_texture(&array_desc)
}
```

### Mipmapped Texture

Create a texture with multiple mip levels:

```rust
fn create_mipmapped_texture(gpu: &GpuController, width: u32, height: u32) -> wgpu::Texture {
    let mip_levels = (width.max(height) as f32).log2().floor() as u32 + 1;
    
    let mip_desc = TextureDescriptor {
        label: Some("Mipmapped Texture"),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: mip_levels,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };
    
    gpu.create_texture(&mip_desc)
}
```

---

## Command Recording and Submission

### Basic Command Recording

```rust
async fn basic_command_example(gpu: &GpuController) -> anyhow::Result<()> {
    // Create command encoder
    let mut encoder = gpu.create_command_encoder("Basic Commands");
    
    // Record some commands (example with buffer copy)
    // encoder.copy_buffer_to_buffer(&src_buffer, 0, &dst_buffer, 0, size);
    
    // Submit to GPU
    gpu.submit(encoder);
    
    println!("Commands submitted to GPU");
    Ok(())
}
```

### Multiple Command Buffers

```rust
async fn multiple_commands_example(gpu: &GpuController) -> anyhow::Result<()> {
    // Create multiple encoders for different tasks
    let shadow_encoder = gpu.create_command_encoder("Shadow Pass");
    let main_encoder = gpu.create_command_encoder("Main Pass");
    let post_encoder = gpu.create_command_encoder("Post Processing");
    
    // Record commands for each pass
    // ... record shadow pass commands
    // ... record main pass commands  
    // ... record post processing commands
    
    // Submit in order
    gpu.submit(shadow_encoder);
    gpu.submit(main_encoder);
    gpu.submit(post_encoder);
    
    Ok(())
}
```

### Timed Command Execution

```rust
use std::time::Instant;

async fn timed_command_example(gpu: &GpuController) -> anyhow::Result<()> {
    let start = Instant::now();
    
    let mut encoder = gpu.create_command_encoder("Timed Commands");
    
    // Record time-sensitive commands
    // ... record commands
    
    gpu.submit(encoder);
    
    let duration = start.elapsed();
    println!("Command recording took: {:?}", duration);
    
    Ok(())
}
```

---

## Render Passes

### Basic Render Pass

```rust
use wgpu::{RenderPassDescriptor, RenderPassColorAttachment, Operations, LoadOp, StoreOp, Color, TextureViewDescriptor};

fn basic_render_pass(gpu: &GpuController, render_target: &wgpu::Texture) {
    let mut encoder = gpu.create_command_encoder("Basic Render Pass");
    
    {
        let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &render_target.create_view(&TextureViewDescriptor::default()),
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        
        // Render commands would go here
        // render_pass.set_pipeline(&render_pipeline);
        // render_pass.set_bind_group(0, &bind_group, &[]);
        // render_pass.draw(0..3, 0..1);
    }
    
    gpu.submit(encoder);
}
```

### Render Pass with Depth Testing

```rust
use wgpu::{RenderPassDepthStencilAttachment, CompareFunction};

fn render_pass_with_depth(
    gpu: &GpuController, 
    color_target: &wgpu::Texture,
    depth_target: &wgpu::Texture
) {
    let mut encoder = gpu.create_command_encoder("Depth Render Pass");
    
    {
        let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("3D Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &color_target.create_view(&TextureViewDescriptor::default()),
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &depth_target.create_view(&TextureViewDescriptor::default()),
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        
        // 3D rendering commands
        // render_pass.set_pipeline(&pipeline_3d);
        // render_pass.set_bind_group(0, &camera_bind_group, &[]);
        // render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
    }
    
    gpu.submit(encoder);
}
```

### Multi-target Render Pass

```rust
fn multi_target_render_pass(
    gpu: &GpuController,
    color_target: &wgpu::Texture,
    normal_target: &wgpu::Texture,
    depth_target: &wgpu::Texture
) {
    let mut encoder = gpu.create_command_encoder("G-Buffer Pass");
    
    {
        let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("G-Buffer Generation"),
            color_attachments: &[
                // Color attachment
                Some(RenderPassColorAttachment {
                    view: &color_target.create_view(&TextureViewDescriptor::default()),
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                }),
                // Normal attachment
                Some(RenderPassColorAttachment {
                    view: &normal_target.create_view(&TextureViewDescriptor::default()),
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color { r: 0.5, g: 0.5, b: 1.0, a: 1.0 }),
                        store: StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &depth_target.create_view(&TextureViewDescriptor::default()),
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        
        // Deferred rendering geometry pass
        // render_pass.set_pipeline(&gbuffer_pipeline);
        // ... render geometry to multiple targets
    }
    
    gpu.submit(encoder);
}
```

---

## Compute Shaders

### Basic Compute Dispatch

```rust
use wgpu::{ComputePassDescriptor, TextureUsages};

async fn basic_compute_example(gpu: &GpuController) -> anyhow::Result<()> {
    // Create storage texture for compute
    let storage_texture = gpu.create_texture(&TextureDescriptor {
        label: Some("Compute Storage"),
        size: Extent3d {
            width: 512,
            height: 512,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    
    let mut encoder = gpu.create_command_encoder("Compute Pass");
    
    {
        let compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Image Processing"),
            timestamp_writes: None,
        });
        
        // Dispatch compute work
        // compute_pass.set_pipeline(&compute_pipeline);
        // compute_pass.set_bind_group(0, &compute_bind_group, &[]);
        // compute_pass.dispatch_workgroups(512 / 16, 512 / 16, 1);
    }
    
    gpu.submit(encoder);
    
    Ok(())
}
```

### Multi-stage Compute Pipeline

```rust
async fn multi_stage_compute(gpu: &GpuController) -> anyhow::Result<()> {
    let mut encoder = gpu.create_command_encoder("Multi-stage Compute");
    
    // Stage 1: Preprocessing
    {
        let compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Preprocessing"),
            timestamp_writes: None,
        });
        
        // Preprocessing compute work
        // compute_pass.set_pipeline(&preprocess_pipeline);
        // compute_pass.dispatch_workgroups(64, 64, 1);
    }
    
    // Stage 2: Main computation
    {
        let compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Main Computation"),
            timestamp_writes: None,
        });
        
        // Main compute work
        // compute_pass.set_pipeline(&main_compute_pipeline);
        // compute_pass.dispatch_workgroups(128, 128, 1);
    }
    
    // Stage 3: Postprocessing
    {
        let compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Postprocessing"),
            timestamp_writes: None,
        });
        
        // Postprocessing compute work
        // compute_pass.set_pipeline(&postprocess_pipeline);
        // compute_pass.dispatch_workgroups(32, 32, 1);
    }
    
    gpu.submit(encoder);
    
    Ok(())
}
```

---

## Multi-threaded Usage

### Parallel Command Recording

```rust
use std::sync::Arc;
use std::thread;

async fn parallel_command_recording(gpu: Arc<GpuController>) -> anyhow::Result<()> {
    let mut handles = vec![];
    
    // Spawn multiple threads for parallel work
    for i in 0..4 {
        let gpu_clone = Arc::clone(&gpu);
        let handle = thread::spawn(move || {
            let encoder = gpu_clone.create_command_encoder(&format!("Thread {} Work", i));
            
            // Record thread-specific commands
            // ... record commands
            
            gpu_clone.submit(encoder);
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("All parallel work completed");
    Ok(())
}
```

### Async Task Spawning

```rust
use tokio::task;

async fn async_task_example(gpu: Arc<GpuController>) -> anyhow::Result<()> {
    let mut tasks = vec![];
    
    // Spawn async tasks
    for i in 0..8 {
        let gpu_clone = Arc::clone(&gpu);
        let task = task::spawn(async move {
            let encoder = gpu_clone.create_command_encoder(&format!("Async Task {}", i));
            
            // Async work simulation
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            
            // Record and submit commands
            gpu_clone.submit(encoder);
        });
        tasks.push(task);
    }
    
    // Wait for all tasks
    for task in tasks {
        task.await?;
    }
    
    Ok(())
}
```

### Thread-safe Configuration Access

```rust
use std::sync::Arc;
use std::thread;

fn threaded_config_access(gpu: Arc<GpuController>) -> anyhow::Result<()> {
    let mut handles = vec![];
    
    for i in 0..3 {
        let gpu_clone = Arc::clone(&gpu);
        let handle = thread::spawn(move || -> anyhow::Result<()> {
            // Safe access to surface configuration
            let dimensions = gpu_clone.with_surface_config(|config| {
                (config.width, config.height, config.format)
            })?;
            
            println!("Thread {}: Surface {}x{}, format: {:?}", 
                     i, dimensions.0, dimensions.1, dimensions.2);
            
            Ok(())
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap()?;
    }
    
    Ok(())
}
```

---

## Error Handling

### Graceful Initialization Fallback

```rust
async fn robust_initialization() -> anyhow::Result<Arc<GpuController>> {
    // Try high-end features first
    if let Ok(gpu) = GpuController::new(
        Some(Features::all()),
        None,
        None
    ).await {
        println!("Initialized with all features");
        return Ok(gpu);
    }
    
    // Fall back to essential features
    let essential_features = Features::TEXTURE_BINDING_ARRAY;
    if let Ok(gpu) = GpuController::new(
        Some(essential_features),
        None,
        None
    ).await {
        println!("Initialized with essential features");
        return Ok(gpu);
    }
    
    // Fall back to defaults
    match GpuController::new(None, None, None).await {
        Ok(gpu) => {
            println!("Initialized with default features");
            Ok(gpu)
        },
        Err(e) => {
            eprintln!("Failed to initialize GPU: {}", e);
            Err(e)
        }
    }
}
```

### Error Recovery Patterns

```rust
async fn error_recovery_example(gpu: &GpuController) -> anyhow::Result<()> {
    // Attempt risky operation with recovery
    match gpu.with_surface_config(|config| config.width) {
        Ok(width) => {
            println!("Surface width: {}", width);
        },
        Err(e) => {
            eprintln!("Failed to access surface config: {}", e);
            // Attempt recovery or use fallback
            println!("Using fallback width: 1920");
        }
    }
    
    Ok(())
}
```

### Comprehensive Error Handling

```rust
use anyhow::{Context, Result};

async fn comprehensive_error_handling() -> Result<()> {
    let gpu = GpuController::new(None, None, None).await
        .context("Failed to initialize GPU controller")?;
    
    // Create texture with error context
    let _texture = gpu.create_texture(&TextureDescriptor {
        label: Some("Test Texture"),
        size: Extent3d { width: 1024, height: 1024, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    
    // Access configuration with error handling
    let _format = gpu.with_surface_config(|config| config.format)
        .context("Failed to access surface configuration")?;
    
    println!("All operations completed successfully");
    Ok(())
}
```

---

## Performance Optimization

### Efficient Resource Reuse

```rust
struct ResourcePool {
    gpu: Arc<GpuController>,
    linear_sampler: wgpu::Sampler,
    nearest_sampler: wgpu::Sampler,
}

impl ResourcePool {
    fn new(gpu: Arc<GpuController>) -> Self {
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
        
        Self {
            gpu,
            linear_sampler,
            nearest_sampler,
        }
    }
    
    fn get_linear_sampler(&self) -> &wgpu::Sampler {
        &self.linear_sampler
    }
    
    fn get_nearest_sampler(&self) -> &wgpu::Sampler {
        &self.nearest_sampler
    }
}
```

### Batch Command Submission

```rust
struct CommandBatch {
    gpu: Arc<GpuController>,
    encoders: Vec<wgpu::CommandEncoder>,
}

impl CommandBatch {
    fn new(gpu: Arc<GpuController>) -> Self {
        Self {
            gpu,
            encoders: Vec::new(),
        }
    }
    
    fn add_work<F>(&mut self, label: &str, work: F) 
    where
        F: FnOnce(&mut wgpu::CommandEncoder),
    {
        let mut encoder = self.gpu.create_command_encoder(label);
        work(&mut encoder);
        self.encoders.push(encoder);
    }
    
    fn submit_all(self) {
        for encoder in self.encoders {
            self.gpu.submit(encoder);
        }
    }
}

// Usage
async fn batch_example(gpu: Arc<GpuController>) -> anyhow::Result<()> {
    let mut batch = CommandBatch::new(gpu);
    
    batch.add_work("Shadow Pass", |encoder| {
        // Record shadow pass commands
    });
    
    batch.add_work("Main Pass", |encoder| {
        // Record main pass commands
    });
    
    batch.add_work("Post Process", |encoder| {
        // Record post-processing commands
    });
    
    batch.submit_all();
    
    Ok(())
}
```

---

## Real-world Scenarios

### Simple 2D Renderer

```rust
struct Simple2DRenderer {
    gpu: Arc<GpuController>,
    render_target: wgpu::Texture,
    quad_sampler: wgpu::Sampler,
}

impl Simple2DRenderer {
    fn new(gpu: Arc<GpuController>, width: u32, height: u32) -> Self {
        let render_target = gpu.create_texture(&TextureDescriptor {
            label: Some("2D Render Target"),
            size: Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        
        let quad_sampler = gpu.create_sampler(&SamplerDescriptor {
            label: Some("2D Quad Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });
        
        Self {
            gpu,
            render_target,
            quad_sampler,
        }
    }
    
    fn render_frame(&self) {
        let mut encoder = self.gpu.create_command_encoder("2D Frame");
        
        {
            let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("2D Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self.render_target.create_view(&TextureViewDescriptor::default()),
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            
            // 2D rendering commands would go here
            // render_pass.set_pipeline(&sprite_pipeline);
            // render_pass.set_bind_group(0, &sprite_bind_group, &[]);
            // render_pass.draw(0..6, 0..sprite_count);
        }
        
        self.gpu.submit(encoder);
    }
}
```

### Particle System

```rust
struct ParticleSystem {
    gpu: Arc<GpuController>,
    particle_buffer: wgpu::Texture,
    velocity_buffer: wgpu::Texture,
}

impl ParticleSystem {
    fn new(gpu: Arc<GpuController>, particle_count: u32) -> Self {
        let buffer_size = (particle_count as f32).sqrt() as u32;
        
        let particle_buffer = gpu.create_texture(&TextureDescriptor {
            label: Some("Particle Positions"),
            size: Extent3d {
                width: buffer_size,
                height: buffer_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        
        let velocity_buffer = gpu.create_texture(&TextureDescriptor {
            label: Some("Particle Velocities"),
            size: Extent3d {
                width: buffer_size,
                height: buffer_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        
        Self {
            gpu,
            particle_buffer,
            velocity_buffer,
        }
    }
    
    fn update(&self, delta_time: f32) {
        let mut encoder = self.gpu.create_command_encoder("Particle Update");
        
        {
            let compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Particle Physics"),
                timestamp_writes: None,
            });
            
            // Particle update compute work
            // compute_pass.set_pipeline(&particle_update_pipeline);
            // compute_pass.set_bind_group(0, &particle_bind_group, &[]);
            // compute_pass.dispatch_workgro