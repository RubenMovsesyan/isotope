use std::sync::{RwLock, RwLockReadGuard};

use anyhow::Result;
use log::*;
use pollster::FutureExt;
use wgpu::{
    Adapter, Backends, CompositeAlphaMode, Device, DeviceDescriptor, Features, FeaturesWGPU,
    Instance, InstanceDescriptor, Limits, MemoryHints, PolygonMode, PowerPreference, PresentMode,
    Queue, RenderPipeline, RequestAdapterOptionsBase, SurfaceConfiguration, TextureFormat,
    TextureUsages, Trace, include_wgsl,
};

use crate::{
    construct_render_pipeline,
    element::{buffered::Buffered, model::ModelInstance, model_vertex::ModelVertex},
    photon::renderer::photon_layouts::PhotonLayoutsManager,
};

#[derive(Debug)]
pub struct GpuController {
    pub(crate) instance: Instance,
    pub(crate) adapter: Adapter,
    pub(crate) device: Device,
    pub(crate) queue: Queue,

    pub(crate) layouts: PhotonLayoutsManager,

    // Interior Mutability
    pub(crate) surface_configuration: RwLock<SurfaceConfiguration>,
    pub(crate) default_render_pipeline: RenderPipeline,
}

impl GpuController {
    pub(crate) fn new() -> Result<Self> {
        info!("Initializeing WGPU");
        // Initialize WGPU
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&RequestAdapterOptionsBase {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .block_on()?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("Device and Queue"),
                required_features: Features {
                    features_wgpu: FeaturesWGPU::POLYGON_MODE_LINE,
                    ..Default::default()
                },
                required_limits: Limits {
                    max_bind_groups: 8,
                    ..Default::default()
                },
                memory_hints: MemoryHints::default(),
                trace: Trace::default(),
            })
            .block_on()?;

        info!("WGPU Initialized");

        let layouts = PhotonLayoutsManager::new(&device);
        info!("Layouts Initialized");

        let surface_configuration = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Rgba8UnormSrgb,
            width: 1,
            height: 1,
            present_mode: PresentMode::AutoNoVsync,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // Load in the shaders
        let vertex_shader =
            device.create_shader_module(include_wgsl!("../photon/renderer/shaders/vert.wgsl"));
        let fragment_shader =
            device.create_shader_module(include_wgsl!("../photon/renderer/shaders/frag.wgsl"));

        // Create the default render pipeline
        let default_render_pipeline = construct_render_pipeline!(
            &device,
            &surface_configuration,
            vertex_shader,
            fragment_shader,
            String::from("Photon"),
            PolygonMode::Fill,
            &[ModelVertex::desc(), ModelInstance::desc()],
            &layouts.camera_layout,
            &layouts.lights_layout,
            &layouts.texture_layout,
            &layouts.model_layout,
            &layouts.material_layout
        );

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            layouts,
            surface_configuration: RwLock::new(surface_configuration),
            default_render_pipeline,
        })
    }

    pub(crate) fn surface_configuration(&self) -> RwLockReadGuard<SurfaceConfiguration> {
        self.surface_configuration.read().unwrap() // TODO: Fix
    }

    // pub(crate) fn default_render_pipeline(&self) -> RwLockReadGuard<RenderPipeline> {
    //     self.default_render_pipeline.read().unwrap() // TODO: Fix
    // }
}
