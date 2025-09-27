//! # Default Configuration Module
//!
//! This module provides sensible default configurations for WGPU components,
//! particularly surface configurations that are commonly used across different
//! graphics applications.
//!
//! The defaults are chosen to be:
//! - **Compatible** with most hardware and drivers
//! - **Performant** for typical use cases
//! - **Safe** to use without additional configuration
//! - **Flexible** enough to serve as a starting point for customization

use wgpu::{CompositeAlphaMode, PresentMode, SurfaceConfiguration, TextureFormat, TextureUsages};

/// Default surface configuration for WGPU rendering contexts.
///
/// This configuration provides a sensible starting point for most graphics applications.
/// It's designed to work reliably across different platforms and hardware configurations
/// while providing good performance characteristics.
///
/// ## Configuration Details
///
/// - **Usage**: `RENDER_ATTACHMENT` - Optimized for rendering operations
/// - **Format**: `Rgba8UnormSrgb` - Standard 8-bit RGBA with sRGB color space
/// - **Dimensions**: 1x1 pixels - Minimal size, should be resized for actual use
/// - **Present Mode**: `AutoNoVsync` - Automatic presentation without forced vsync
/// - **Alpha Mode**: `Auto` - Automatic alpha channel handling
/// - **View Formats**: Empty - No additional view formats enabled
/// - **Frame Latency**: 2 frames - Balanced latency and performance
///
/// ## Usage Example
///
/// ```rust,no_run
/// use gpu_controller::defaults::DEFAULT_SURFACE_CONFIGURATION;
/// use wgpu::SurfaceConfiguration;
///
/// // Use the default configuration
/// let mut config = DEFAULT_SURFACE_CONFIGURATION;
///
/// // Customize as needed
/// config.width = 1920;
/// config.height = 1080;
/// ```
///
/// ## Field Explanations
///
/// ### Usage: TextureUsages::RENDER_ATTACHMENT
/// Specifies that this surface will be used as a render target. This is the most
/// common usage pattern for display surfaces.
///
/// ### Format: TextureFormat::Rgba8UnormSrgb
/// Uses 8 bits per channel (red, green, blue, alpha) with automatic sRGB gamma
/// correction. This format is widely supported and provides good color accuracy
/// for typical display content.
///
/// ### Dimensions: 1x1
/// Minimal size to ensure valid configuration. Applications should resize this
/// to match their actual window or viewport dimensions.
///
/// ### Present Mode: PresentMode::AutoNoVsync
/// Allows the system to choose the best presentation mode while avoiding forced
/// vertical synchronization. This provides a good balance between performance
/// and visual quality.
///
/// ### Alpha Mode: CompositeAlphaMode::Auto
/// Lets the system automatically handle alpha channel composition with the
/// desktop compositor or window system.
///
/// ### View Formats: Empty Vec
/// No additional texture view formats are enabled by default. Applications
/// requiring specific view formats should modify this field.
///
/// ### Frame Latency: 2 frames
/// Allows up to 2 frames to be queued for presentation, providing a balance
/// between input responsiveness and rendering smoothness.
pub(crate) const DEFAULT_SURFACE_CONFIGURATION: SurfaceConfiguration = SurfaceConfiguration {
    usage: TextureUsages::RENDER_ATTACHMENT,
    format: TextureFormat::Rgba8UnormSrgb,
    width: 1,
    height: 1,
    present_mode: PresentMode::AutoNoVsync,
    alpha_mode: CompositeAlphaMode::Auto,
    view_formats: vec![],
    desired_maximum_frame_latency: 2,
};

pub const VERTECIES_BUFFER_INDEX: u32 = 0;
pub const INSTANCE_BUFFER_INDEX: u32 = 1;
