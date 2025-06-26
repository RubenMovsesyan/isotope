use wgpu::{Adapter, Device, Instance, Queue};

mod layouts;

#[derive(Debug)]
pub struct GpuController {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}
