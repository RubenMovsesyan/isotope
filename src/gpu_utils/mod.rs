use anyhow::Result;
use log::*;
use pollster::FutureExt;
use wgpu::{
    Adapter, Backends, Device, DeviceDescriptor, Features, FeaturesWGPU, Instance,
    InstanceDescriptor, Limits, MemoryHints, PowerPreference, Queue, RequestAdapterOptionsBase,
    Trace,
};

#[derive(Debug)]
pub struct GpuController {
    pub(crate) instance: Instance,
    pub(crate) adapter: Adapter,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
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
                    max_bind_groups: 5,
                    ..Default::default()
                },
                memory_hints: MemoryHints::default(),
                trace: Trace::default(),
            })
            .block_on()?;

        info!("WGPU Initialized");

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }
}
