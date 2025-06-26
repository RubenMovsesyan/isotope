use std::collections::HashMap;

use anyhow::{Result, anyhow};
use wgpu::{BindGroupLayout, BindGroupLayoutDescriptor, Device};

// Helper functions
#[inline]
fn get_descriptor_label(descriptor: &BindGroupLayoutDescriptor) -> Result<String> {
    if let Some(label) = descriptor.label {
        Ok(label.to_string())
    } else {
        Err(anyhow!("Descriptor Has No Label"))
    }
}

#[derive(Debug)]
pub(crate) struct LayoutsManager {
    layouts: HashMap<String, BindGroupLayout>,
}

impl LayoutsManager {
    pub(crate) fn new() -> Self {
        Self {
            layouts: HashMap::new(),
        }
    }

    pub(crate) fn get_layout_from_desc(
        &mut self,
        device: &Device,
        descriptor: &BindGroupLayoutDescriptor,
    ) -> Result<&BindGroupLayout> {
        let label = get_descriptor_label(descriptor)?;

        Ok(self
            .layouts
            .entry(label)
            .or_insert(device.create_bind_group_layout(descriptor)))
    }

    pub(crate) fn get_layout_from_label(&self, label: &str) -> Result<&BindGroupLayout> {
        self.layouts
            .get(label)
            .ok_or_else(|| anyhow!(format!("Layout {} does not exist", label)))
    }
}
