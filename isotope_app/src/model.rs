use std::{fs::File, path::Path};

use anyhow::Result;
use gpu_controller::Mesh;
use log::{debug, info};
use matter_vault::SharedMatter;

use crate::asset_server::AssetServer;

pub struct Model {
    meshes: Vec<SharedMatter<Mesh>>,
}

impl Model {
    pub fn from_obj<P>(path: P, asset_server: &AssetServer) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        debug!("Full Path {:#?}", path.as_ref());

        // TODO: Implement crate for IO
        todo!()
    }
}
