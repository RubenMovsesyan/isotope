use wgpu::{CompositeAlphaMode, PresentMode, SurfaceConfiguration, TextureFormat, TextureUsages};

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
