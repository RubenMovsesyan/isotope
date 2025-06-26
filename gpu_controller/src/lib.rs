use std::sync::{Arc, RwLock};

use anyhow::Result;
use defaults::DEFAULT_SURFACE_CONFIGURATION;
use layouts::LayoutsManager;
use log::info;
use wgpu::{
    Adapter, Backends, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits,
    MemoryHints, PowerPreference, Queue, RequestAdapterOptionsBase, SurfaceConfiguration, Trace,
};

mod defaults;
mod layouts;

#[derive(Debug)]
pub struct GpuController {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,

    layouts_manager: LayoutsManager,

    // Interior Mutability
    surface_configuration: RwLock<SurfaceConfiguration>,
}

impl GpuController {
    pub async fn new(
        required_features: Option<Features>,
        required_limits: Option<Limits>,
        surface_configuration: Option<SurfaceConfiguration>,
    ) -> Result<Arc<Self>> {
        info!("Initializing WGPU");

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

        Ok(Arc::new(Self {
            instance,
            adapter,
            device,
            queue,
            layouts_manager,
            surface_configuration,
        }))
    }
}
