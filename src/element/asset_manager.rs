use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, RwLock},
};

use crate::{gpu_utils::GpuController, photon::renderer::texture::PhotonTexture};

use super::mesh::Mesh;

#[derive(Debug)]
pub(crate) struct SharedAsset<T>(Arc<RwLock<T>>);

unsafe impl<T> Send for SharedAsset<T> {}
unsafe impl<T> Sync for SharedAsset<T> {}

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
    meshes: HashMap<String, SharedAsset<Mesh>>,

    // For loading assets
    pub(crate) gpu_controller: Arc<GpuController>,
}

impl AssetManager {
    pub(crate) fn new(gpu_controller: Arc<GpuController>) -> Self {
        Self {
            textures: HashMap::new(),
            meshes: HashMap::new(),

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
            // return Arc::new(PhotonTexture::new_empty(self.gpu_controller.clone()));
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

    pub(crate) fn get_mesh<P>(&mut self, label: String) -> SharedAsset<Mesh> {
        // If the texture path cannot be read for some reason just return an empty texture
        if let Some(mesh) = self.meshes.get(&label) {
            mesh.clone()
        } else {
            // let new_texture = SharedAsset::new(
            //     if let Ok(texture) =
            //         PhotonTexture::new_from_path(self.gpu_controller.clone(), &label)
            //     {
            //         texture
            //     } else {
            //         PhotonTexture::new_empty(self.gpu_controller.clone())
            //     },
            // );

            // self.textures.insert(label, new_texture.clone());

            // new_texture
            todo!()
        }
    }
}
