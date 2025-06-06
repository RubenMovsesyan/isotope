use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, RwLock},
};

use log::*;

use crate::{gpu_utils::GpuController, photon::renderer::texture::PhotonTexture};

use super::material::Material;

#[derive(Debug)]
pub(crate) struct SharedAsset<T>(Arc<RwLock<T>>);

unsafe impl<T> Send for SharedAsset<T> {}
unsafe impl<T> Sync for SharedAsset<T> {}

#[allow(dead_code)]
impl<T> SharedAsset<T> {
    pub(crate) fn new(t: T) -> Self {
        Self(Arc::new(RwLock::new(t)))
    }

    pub(crate) fn with_read<F, R>(&self, callback: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        if let Ok(asset) = self.0.read() {
            callback(&asset)
        } else {
            unimplemented!();
        }
    }

    pub(crate) fn with_write<F, R>(&self, callback: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        if let Ok(mut asset) = self.0.write() {
            callback(&mut asset)
        } else {
            unimplemented!();
        }
    }
}

impl<T> Clone for SharedAsset<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(Debug)]
pub struct AssetManager {
    textures: HashMap<String, SharedAsset<PhotonTexture>>,
    materials: HashMap<String, SharedAsset<Material>>,

    // For loading assets
    pub(crate) gpu_controller: Arc<GpuController>,
}

impl AssetManager {
    pub(crate) fn new(gpu_controller: Arc<GpuController>) -> Self {
        Self {
            textures: HashMap::new(),
            materials: HashMap::new(),

            gpu_controller,
        }
    }

    pub(crate) fn get_texture<P>(&mut self, texture_path: P) -> SharedAsset<PhotonTexture>
    where
        P: AsRef<Path>,
    {
        // If the texture path cannot be read for some reason just return an empty texture
        let texture_path = if let Some(path) = texture_path.as_ref().to_str() {
            path.to_string()
        } else {
            return SharedAsset::new(PhotonTexture::new_empty(self.gpu_controller.clone()));
        };

        if let Some(texture) = self.textures.get(&texture_path) {
            texture.clone()
        } else {
            let new_texture = SharedAsset::new(
                if let Ok(texture) =
                    PhotonTexture::new_from_path(self.gpu_controller.clone(), &texture_path)
                {
                    texture
                } else {
                    PhotonTexture::new_empty(self.gpu_controller.clone())
                },
            );

            self.textures.insert(texture_path, new_texture.clone());

            new_texture
        }
    }

    pub(crate) fn get_material(&mut self, material: String) -> SharedAsset<Material> {
        if let Some(material) = self.materials.get(&material) {
            material.clone()
        } else {
            let new_material = SharedAsset::new(Material::with_label(material.clone()));

            self.materials.insert(material, new_material.clone());

            new_material
        }
    }

    pub(crate) fn search_material(&self, material: String) -> Option<SharedAsset<Material>> {
        if let Some(material) = self.materials.get(&material) {
            Some(material.clone())
        } else {
            None
        }
    }

    pub(crate) fn add_material(&mut self, material: Material) -> SharedAsset<Material> {
        if let Some(material) = self.materials.get(&material.label()) {
            warn!("Material Already in Shared Assets");
            material.clone()
        } else {
            // First make sure to buffer the material onto the gpu

            let label = material.label();
            let shared_material = SharedAsset::new(material);
            self.materials.insert(label, shared_material.clone());
            shared_material
        }
    }
}
