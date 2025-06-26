use std::{path::Path, sync::Arc};

use log::*;
use obj_loader::Obj;
use wgpu::{Buffer, BufferDescriptor, BufferUsages};

use crate::{
    GpuController,
    element::{material::Material, mesh::Mesh, model::ModelTransform},
    photon::instancer::{InstanceBufferDescriptor, Instancer},
};

use super::{
    super::asset_manager::{AssetManager, SharedAsset},
    ModelInstance,
};

#[allow(dead_code)]
#[derive(Debug)]
pub struct ModelFragment {
    pub(crate) meshes: Vec<Mesh>,
    materials: Vec<SharedAsset<Material>>,

    // GPU
    transform_buffer: Arc<Buffer>,
    gpu_controller: Arc<GpuController>,

    // For Culling
    pub(super) culling_position: [f32; 3],

    // For gpu instancing
    pub(super) instancer: Arc<Instancer<ModelInstance>>,
}

impl ModelFragment {
    pub(crate) fn write_transform(&self, new_transform: ModelTransform) {
        self.gpu_controller.queue.write_buffer(
            &self.transform_buffer,
            0,
            bytemuck::cast_slice(&[new_transform]),
        );
    }

    pub(crate) fn from_obj<P>(path: P, asset_manager: &mut AssetManager) -> Self
    where
        P: AsRef<Path>,
    {
        debug!("Full Path: {:#?}", path.as_ref());
        // if let Ok(mut asset_manager) = asset_manager.write() {
        let gpu_controller = asset_manager.gpu_controller.clone();

        let model_obj = if let Ok(obj) = Obj::new(&path) {
            obj
        } else {
            todo!()
        };

        // Create the transform buffer as it is needed for the meshes to reference
        let transform_buffer = gpu_controller.device.create_buffer(&BufferDescriptor {
            label: Some("Model Transform"),
            mapped_at_creation: false,
            size: std::mem::size_of::<ModelTransform>() as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        let path_parent = if let Some(parent) = path.as_ref().parent() {
            parent
        } else {
            Path::new("")
        };

        // Load all the materials first in case we need some
        let materials = model_obj
            .materials
            .iter()
            .map(|(_, material)| {
                // Search for the material in the asset manager and add it if not in there
                let new_material: Material = material.into();

                if let Some(material) = asset_manager.search_material(new_material.label()) {
                    material
                } else {
                    info!(
                        "Adding New Material to Asset Manager: {}",
                        new_material.label()
                    );
                    let buffered = new_material.buffer(path_parent, asset_manager);
                    asset_manager.add_material(buffered)
                }
            })
            .collect::<Vec<SharedAsset<Material>>>();

        let meshes = model_obj
            .meshes
            .iter()
            .map(|mesh| {
                let new_mesh: Mesh = mesh.into();
                new_mesh.buffer(&transform_buffer, asset_manager)
            })
            .collect::<Vec<Mesh>>();

        let instancer = Arc::new(Instancer::new_series(
            gpu_controller.clone(),
            InstanceBufferDescriptor::Size(1),
            "Mesh",
        ));

        Self {
            gpu_controller,
            meshes,
            materials,
            culling_position: [0.0, 0.0, 0.0],
            transform_buffer: Arc::new(transform_buffer),
            instancer,
        }
    }
}
