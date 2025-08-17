//! # GPU Controller Library
//!
//! A high-level abstraction over WGPU that provides simplified GPU resource management
//! and command submission. This library is designed to streamline common GPU operations
//! while maintaining flexibility for complex graphics applications.
//!
//! ## Features
//!
//! - **Simplified GPU initialization** with automatic adapter selection and device creation
//! - **Thread-safe resource management** using Arc and RwLock patterns
//! - **Efficient bind group layout caching** to minimize redundant GPU object creation
//! - **Command encoder creation and submission** with intuitive API
//! - **Texture and sampler creation** with direct device access
//! - **Flexible configuration** supporting custom features, limits, and surface settings
//!
//! ## Example
//!
//! ```rust,no_run
//! use gpu_controller::GpuController;
//! use std::sync::Arc;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Initialize GPU controller with default settings
//! let gpu = GpuController::new(None, None, None).await?;
//!
//! // Create and submit commands
//! let encoder = gpu.create_command_encoder("My Commands");
//! gpu.submit(encoder);
//! # Ok(())
//! # }
//! ```

const DESIRED_MAX_FRAME_LATENCY: u32 = 2;

use std::{
    borrow::Cow,
    sync::{Arc, RwLock},
};

use anyhow::{Result, anyhow};
use defaults::DEFAULT_SURFACE_CONFIGURATION;
use layouts::LayoutsManager;
use log::info;
use wgpu::{
    Adapter, Backends, BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    Buffer, BufferDescriptor, CommandEncoder, CommandEncoderDescriptor, Device, DeviceDescriptor,
    Features, InstanceDescriptor, Limits, MemoryHints, PipelineLayout, PipelineLayoutDescriptor,
    PowerPreference, PresentMode, Queue, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptionsBase, Sampler, SamplerDescriptor, ShaderModule, ShaderModuleDescriptor,
    ShaderSource, Surface, SurfaceConfiguration, Texture, TextureDescriptor, TextureUsages, Trace,
    VertexBufferLayout,
    util::{BufferInitDescriptor, DeviceExt},
};

// public re-exports
pub use geometry::{instance::Instance, mesh::Mesh, vertex::Vertex};
use winit::window::Window;

mod defaults;
mod geometry;
mod layouts;

/// The main GPU controller that manages all WGPU resources and provides a simplified interface
/// for GPU operations.
///
/// `GpuController` encapsulates the core WGPU components (instance, adapter, device, queue)
/// and provides additional functionality like bind group layout caching and thread-safe
/// surface configuration management.
///
/// ## Thread Safety
///
/// This struct is designed to be shared across threads using `Arc`. The surface configuration
/// uses interior mutability with `RwLock` to allow safe concurrent access.
///
/// ## Resource Management
///
/// - **Instance**: The root WGPU object that manages adapters
/// - **Adapter**: Represents a specific GPU/driver combination
/// - **Device**: The logical connection to the GPU
/// - **Queue**: Used for submitting command buffers to the GPU
/// - **LayoutsManager**: Caches bind group layouts to avoid redundant creation
/// - **SurfaceConfiguration**: Thread-safe configuration for rendering surfaces
#[derive(Debug)]
pub struct GpuController {
    instance: wgpu::Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,

    layouts_manager: LayoutsManager,

    // Interior Mutability
    surface_configuration: RwLock<SurfaceConfiguration>,
}

impl GpuController {
    /// Creates a new GPU controller instance with optional custom configuration.
    ///
    /// This async function initializes the entire WGPU stack, including instance creation,
    /// adapter selection, device creation, and queue setup. It also initializes internal
    /// resource managers.
    ///
    /// ## Arguments
    ///
    /// * `required_features` - Optional WGPU features to enable on the device.
    ///   If `None`, uses default features. Common features include texture binding arrays,
    ///   compute shaders, and various texture formats.
    ///
    /// * `required_limits` - Optional custom limits for the device. If `None`, uses
    ///   default limits. This controls things like maximum texture size, buffer sizes,
    ///   and binding limits.
    ///
    /// * `surface_configuration` - Optional custom surface configuration. If `None`,
    ///   uses a default configuration suitable for basic rendering.
    ///
    /// ## Returns
    ///
    /// Returns `Result<Arc<GpuController>>` - an Arc-wrapped GPU controller for sharing
    /// across threads, or an error if initialization fails.
    ///
    /// ## Errors
    ///
    /// This function can fail if:
    /// - No suitable graphics adapter is found
    /// - The requested features or limits are not supported
    /// - Device creation fails due to driver issues
    /// - System resources are insufficient
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// use gpu_controller::GpuController;
    /// use wgpu::{Features, Limits};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// // Initialize with custom features
    /// let features = Features::TEXTURE_BINDING_ARRAY | Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING;
    /// let gpu = GpuController::new(Some(features), None, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(
        required_features: Option<Features>,
        required_limits: Option<Limits>,
        surface_configuration: Option<SurfaceConfiguration>,
    ) -> Result<Arc<Self>> {
        info!("Initializing WGPU");

        // Initialize WGPU
        let instance = wgpu::Instance::new(&InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&RequestAdapterOptionsBase {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("Device and Queue"),
                required_features: match required_features {
                    Some(features) => features,
                    None => Features::default(),
                },
                required_limits: match required_limits {
                    Some(limits) => limits,
                    None => Limits::default(),
                },
                memory_hints: MemoryHints::default(), // TODO: add optional argument for this
                trace: Trace::default(),              // TODO: add optional argument for this
            })
            .await?;

        info!("WGPU Initialized");

        let layouts_manager = LayoutsManager::new();
        info!("Layouts Initialized");

        let surface_configuration = if let Some(config) = surface_configuration {
            RwLock::new(config)
        } else {
            RwLock::new(DEFAULT_SURFACE_CONFIGURATION)
        };
        info!("Surface Configuration Initialized");

        Ok(Arc::new(Self {
            instance,
            adapter,
            device,
            queue,
            layouts_manager,
            surface_configuration,
        }))
    }

    /// Creates a new command encoder with the specified label.
    ///
    /// Command encoders are used to record GPU commands before submitting them to the queue.
    /// Each encoder can record multiple commands and should be submitted using [`submit`](Self::submit)
    /// when recording is complete.
    ///
    /// ## Arguments
    ///
    /// * `label` - A descriptive label for debugging and profiling. This label will appear
    ///   in graphics debugging tools and error messages.
    ///
    /// ## Returns
    ///
    /// Returns a `CommandEncoder` ready for recording commands.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use gpu_controller::GpuController;
    /// # async fn example(gpu: &GpuController) {
    /// let mut encoder = gpu.create_command_encoder("Render Frame");
    ///
    /// // Record commands here
    /// // let render_pass = encoder.begin_render_pass(...);
    ///
    /// gpu.submit(encoder);
    /// # }
    /// ```
    #[inline]
    pub fn create_command_encoder<S>(&self, label: S) -> CommandEncoder
    where
        S: AsRef<str>,
    {
        self.device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some(label.as_ref()),
            })
    }

    /// Submits a command encoder to the GPU queue for execution.
    ///
    /// This method finalizes the command encoder and submits the recorded commands
    /// to the GPU for execution. The commands will be executed asynchronously by the GPU.
    ///
    /// ## Arguments
    ///
    /// * `encoder` - The command encoder containing the recorded commands. The encoder
    ///   is consumed by this operation and cannot be used after submission.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use gpu_controller::GpuController;
    /// # async fn example(gpu: &GpuController) {
    /// let encoder = gpu.create_command_encoder("My Commands");
    /// // Record commands...
    /// gpu.submit(encoder); // Commands are now queued for GPU execution
    /// # }
    /// ```
    #[inline]
    pub fn submit(&self, encoder: CommandEncoder) {
        self.queue.submit(Some(encoder.finish()));
    }

    /// Provides thread-safe read access to the surface configuration.
    ///
    /// This method uses a callback pattern to safely access the surface configuration
    /// while ensuring thread safety through the internal RwLock. The callback receives
    /// an immutable reference to the surface configuration.
    ///
    /// ## Arguments
    ///
    /// * `callback` - A closure that receives a reference to the surface configuration
    ///   and returns a value of type `R`.
    ///
    /// ## Returns
    ///
    /// Returns `Result<R>` where `R` is the return type of the callback, or an error
    /// if the surface configuration lock cannot be acquired.
    ///
    /// ## Errors
    ///
    /// Returns an error if the surface configuration RwLock is poisoned or cannot
    /// be acquired for reading.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use gpu_controller::GpuController;
    /// # async fn example(gpu: &GpuController) -> anyhow::Result<()> {
    /// // Get surface dimensions
    /// let (width, height) = gpu.with_surface_config(|config| {
    ///     (config.width, config.height)
    /// })?;
    ///
    /// println!("Surface size: {}x{}", width, height);
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_surface_config<F, R>(&self, callback: F) -> Result<R>
    where
        F: FnOnce(&SurfaceConfiguration) -> R,
    {
        if let Ok(surface_config) = self.surface_configuration.read() {
            Ok(callback(&surface_config))
        } else {
            Err(anyhow!("Failed to read surface configuration"))
        }
    }

    pub fn write_surface_config<F, R>(&self, callback: F) -> Result<R>
    where
        F: FnOnce(&mut SurfaceConfiguration) -> R,
    {
        if let Ok(mut surface_config) = self.surface_configuration.write() {
            Ok(callback(&mut surface_config))
        } else {
            Err(anyhow!("Failed to write surface configuration"))
        }
    }

    /// Creates a new texture using the provided descriptor.
    ///
    /// This method provides direct access to the device's texture creation functionality.
    /// Textures are used for storing image data, render targets, and storage buffers.
    ///
    /// ## Arguments
    ///
    /// * `texture_descriptor` - A descriptor specifying the texture properties including
    ///   dimensions, format, usage flags, and mip levels.
    ///
    /// ## Returns
    ///
    /// Returns a `Texture` object that can be used for rendering operations.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use gpu_controller::GpuController;
    /// # use wgpu::{TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, Extent3d};
    /// # async fn example(gpu: &GpuController) {
    /// let texture_desc = TextureDescriptor {
    ///     label: Some("My Texture"),
    ///     size: Extent3d { width: 512, height: 512, depth_or_array_layers: 1 },
    ///     mip_level_count: 1,
    ///     sample_count: 1,
    ///     dimension: TextureDimension::D2,
    ///     format: TextureFormat::Rgba8UnormSrgb,
    ///     usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
    ///     view_formats: &[],
    /// };
    ///
    /// let texture = gpu.create_texture(&texture_desc);
    /// # }
    /// ```
    pub fn create_texture(&self, texture_descriptor: &TextureDescriptor) -> Texture {
        self.device.create_texture(texture_descriptor)
    }

    /// Creates a new sampler using the provided descriptor.
    ///
    /// Samplers define how textures are sampled during rendering, including filtering
    /// modes, addressing modes, and comparison functions.
    ///
    /// ## Arguments
    ///
    /// * `sampler_descriptor` - A descriptor specifying the sampler properties including
    ///   filtering, addressing modes, and comparison settings.
    ///
    /// ## Returns
    ///
    /// Returns a `Sampler` object that can be used in bind groups for texture sampling.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use gpu_controller::GpuController;
    /// # use wgpu::{SamplerDescriptor, FilterMode, AddressMode};
    /// # async fn example(gpu: &GpuController) {
    /// let sampler_desc = SamplerDescriptor {
    ///     label: Some("Linear Sampler"),
    ///     address_mode_u: AddressMode::Repeat,
    ///     address_mode_v: AddressMode::Repeat,
    ///     address_mode_w: AddressMode::Repeat,
    ///     mag_filter: FilterMode::Linear,
    ///     min_filter: FilterMode::Linear,
    ///     mipmap_filter: FilterMode::Linear,
    ///     ..Default::default()
    /// };
    ///
    /// let sampler = gpu.create_sampler(&sampler_desc);
    /// # }
    /// ```
    pub fn create_sampler(&self, sampler_descriptor: &SamplerDescriptor) -> Sampler {
        self.device.create_sampler(sampler_descriptor)
    }

    pub fn create_bind_group_layout(
        &self,
        bind_group_layout_descriptor: &BindGroupLayoutDescriptor,
    ) -> BindGroupLayout {
        self.device
            .create_bind_group_layout(bind_group_layout_descriptor)
    }

    pub fn create_bind_group(&self, bind_group_descriptor: &BindGroupDescriptor) -> BindGroup {
        self.device.create_bind_group(bind_group_descriptor)
    }

    pub fn create_buffer(&self, buffer_descriptor: &BufferDescriptor) -> Buffer {
        self.device.create_buffer(buffer_descriptor)
    }

    pub fn create_buffer_init(&self, buffer_init_descriptor: &BufferInitDescriptor) -> Buffer {
        self.device.create_buffer_init(buffer_init_descriptor)
    }

    pub fn create_pipeline_layout(
        &self,
        pipeline_layout_descriptor: &PipelineLayoutDescriptor,
    ) -> PipelineLayout {
        self.device
            .create_pipeline_layout(pipeline_layout_descriptor)
    }

    pub fn create_render_pipeline(
        &self,
        render_pipeline_descriptor: &RenderPipelineDescriptor,
    ) -> RenderPipeline {
        self.device
            .create_render_pipeline(render_pipeline_descriptor)
    }

    pub fn create_shader(&self, shader: &str) -> ShaderModule {
        self.device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(shader)),
        })
    }

    pub fn create_surface(&self, window: Arc<Window>) -> Result<Surface<'static>> {
        let surface = self
            .instance
            .create_surface(window.clone())
            .map_err(|e| anyhow!("Failed to create surface: {}", e))?;

        let surface_capabilities = surface.get_capabilities(&self.adapter);
        let size = window.inner_size();

        let surface_format = surface_capabilities
            .formats
            .iter()
            .find(|texture_format| texture_format.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        // Overwrite the surface configuration with the new settings.
        if let Ok(mut sc) = self.surface_configuration.write() {
            *sc = SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.width,
                height: size.height,
                present_mode: PresentMode::AutoNoVsync,
                alpha_mode: surface_capabilities.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: DESIRED_MAX_FRAME_LATENCY,
            };

            surface.configure(&self.device, &sc);
        }

        Ok(surface)
    }

    pub fn configure_surface(&self, surface: &Surface<'static>) {
        if let Ok(sc) = self.surface_configuration.read() {
            surface.configure(&self.device, &sc);
        }
    }
}

pub trait Buffered {
    fn desc() -> VertexBufferLayout<'static>;
}

/// # Tests
///
/// Unit tests for the GPU controller functionality.
#[cfg(test)]
mod test {
    use super::*;
    use smol::block_on;

    /// Tests basic GPU controller initialization with default settings.
    ///
    /// This test verifies that the GPU controller can be successfully created
    /// with all default parameters on the current system.
    #[test]
    fn test_create_gpu_controller() {
        assert!(block_on(GpuController::new(None, None, None)).is_ok());
    }
}
