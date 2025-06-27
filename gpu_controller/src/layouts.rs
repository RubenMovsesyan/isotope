//! # Bind Group Layout Management Module
//!
//! This module provides efficient caching and management of WGPU bind group layouts.
//! Bind group layouts define the structure of resources (textures, samplers, buffers)
//! that shaders can access, and creating them can be expensive. This module implements
//! a caching system to avoid redundant layout creation.
//!
//! ## Key Features
//!
//! - **Automatic Caching**: Layouts are cached by their label to prevent duplicate creation
//! - **Label-Based Retrieval**: Fast HashMap-based lookup for previously created layouts
//! - **Memory Efficient**: Reuses existing layouts instead of creating duplicates
//! - **Error Handling**: Comprehensive error reporting for missing labels and layouts
//!
//! ## Performance Benefits
//!
//! Creating bind group layouts involves GPU driver communication and validation,
//! which can be costly when done repeatedly. By caching layouts based on their
//! descriptors' labels, this module provides significant performance improvements
//! for applications that use the same layout configurations multiple times.
//!
//! ## Usage Pattern
//!
//! The typical usage pattern involves:
//! 1. Creating a layout descriptor with a unique label
//! 2. Calling `get_layout_from_desc` to get or create the layout
//! 3. Using `get_layout_from_label` for subsequent retrievals
//!
//! ## Thread Safety
//!
//! This module is designed to be used within the `GpuController` which handles
//! thread safety at a higher level. The `LayoutsManager` itself is not thread-safe
//! and should be protected by the controller's synchronization mechanisms.

use std::collections::HashMap;

use anyhow::{Result, anyhow};
use wgpu::{BindGroupLayout, BindGroupLayoutDescriptor, Device};

// ============== Helper Functions ==============
/// Extracts the label from a BindGroupLayoutDescriptor.
///
/// # Arguments
/// * `descriptor` - The bind group layout descriptor to extract the label from
///
/// # Returns
/// * `Ok(String)` - The label as a string if present
/// * `Err` - An error if the descriptor has no label
///
/// # Errors
/// Returns an error if the descriptor's label field is None
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
    /// Creates a new empty LayoutsManager.
    ///
    /// # Returns
    /// A new LayoutsManager instance with an empty layouts HashMap
    pub(crate) fn new() -> Self {
        Self {
            layouts: HashMap::new(),
        }
    }

    /// Gets or creates a BindGroupLayout from a descriptor.
    ///
    /// This method implements a cache pattern - if a layout with the same label
    /// already exists, it returns the cached version. Otherwise, it creates a new
    /// layout using the provided device and descriptor, caches it, and returns it.
    ///
    /// # Arguments
    /// * `device` - The WGPU device used to create the bind group layout
    /// * `descriptor` - The bind group layout descriptor containing the layout specification
    ///
    /// # Returns
    /// * `Ok(&BindGroupLayout)` - A reference to the cached or newly created bind group layout
    /// * `Err` - An error if the descriptor has no label or layout creation fails
    ///
    /// # Errors
    /// Returns an error if:
    /// - The descriptor has no label (required for caching)
    /// - The device fails to create the bind group layout
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

    /// Retrieves a previously cached BindGroupLayout by its label.
    ///
    /// # Arguments
    /// * `label` - The string label of the bind group layout to retrieve
    ///
    /// # Returns
    /// * `Ok(&BindGroupLayout)` - A reference to the cached bind group layout
    /// * `Err` - An error if no layout with the given label exists
    ///
    /// # Errors
    /// Returns an error if no layout with the specified label has been cached
    pub(crate) fn get_layout_from_label(&self, label: &str) -> Result<&BindGroupLayout> {
        self.layouts
            .get(label)
            .ok_or_else(|| anyhow!(format!("Layout {} does not exist", label)))
    }
}
