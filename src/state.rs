use std::{any::Any, fmt::Debug, sync::Arc, time::Instant};

use winit::window::Window;

use crate::{Element, PhotonCamera, photon::renderer::PhotonRenderer};

pub trait IsotopeState: Debug + Send + Sync {
    #[allow(unused_variables)]
    fn update(&mut self, delta_t: &Instant) {}

    #[allow(unused_variables)]
    fn render_elements(&self) -> &[Arc<dyn Element>] {
        &[]
    }

    #[allow(unused_variables)]
    fn update_with_camera(&mut self, camera: &mut PhotonCamera, delta_t: &Instant) {}

    #[allow(unused_variables)]
    fn update_with_window(&mut self, window: &Window, delta_t: &Instant) {}

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}
