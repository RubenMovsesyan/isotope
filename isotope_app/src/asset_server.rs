use std::sync::Arc;

use gpu_controller::GpuController;
use matter_vault::MatterVault;

pub struct AssetServer {
    pub(crate) asset_manager: Arc<MatterVault>,
    pub(crate) gpu_controller: Arc<GpuController>,
}
