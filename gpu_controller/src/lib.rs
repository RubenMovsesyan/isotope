use wgpu::{Adapter, Device, Instance, Queue};

#[derive(Debug)]
pub struct GpuController {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}
