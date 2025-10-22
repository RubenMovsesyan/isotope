use std::{ops::Range, sync::Arc};

use gpu_controller::{GpuController, Instance, ShaderModule};

use crate::AssetServer;

pub struct Instancer {
    pub(crate) range: Option<Range<u64>>,
    pub(crate) instancer_kind: InstancerKind,
}

pub(crate) enum InstancerKind {
    Serial {
        serial_modifier: fn(&mut [Instance]),
    },
    Parallel {
        shader: ShaderModule,
    },
}

impl Instancer {
    pub fn new_serial(range: Option<Range<u64>>, serial_modifier: fn(&mut [Instance])) -> Self {
        Self {
            range,
            instancer_kind: InstancerKind::Serial { serial_modifier },
        }
    }

    pub fn new_parallel(
        range: Option<Range<u64>>,
        asset_server: &AssetServer,
        shader: &str,
    ) -> Self {
        todo!()
    }
}
