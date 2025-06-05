use std::{collections::HashMap, path::Path, sync::Arc};

use crate::{gpu_utils::GpuController, photon::renderer::texture::PhotonTexture};

#[derive(Debug)]
pub struct AssetManager {
    textures: HashMap<String, Arc<PhotonTexture>>,

    // For loading assets
    pub(crate) gpu_controller: Arc<GpuController>,
}

impl AssetManager {
    pub(crate) fn new(gpu_controller: Arc<GpuController>) -> Self {
        Self {
            textures: HashMap::new(),

            gpu_controller,
        }
    }

    pub(crate) fn get_texture<P>(&mut self, texture_path: P) -> Arc<PhotonTexture>
    where
        P: AsRef<Path>,
    {
        // If the texture path cannot be read for some reason just return an empty texture
        let texture_path = if let Some(path) = texture_path.as_ref().to_str() {
            path.to_string()
        } else {
            return Arc::new(PhotonTexture::new_empty(self.gpu_controller.clone()));
        };

        if let Some(texture) = self.textures.get(&texture_path) {
            texture.clone()
        } else {
            let new_texture = Arc::new(
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
}
