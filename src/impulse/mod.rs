use std::fmt::Debug;

use winit::keyboard::KeyCode;

use crate::Isotope;

pub type KeyIsPressed = fn(KeyCode, &mut Isotope);

#[derive(Debug, Default)]
pub struct ImpulseManager {
    pub(crate) key_is_pressed: Option<KeyIsPressed>,
}

impl ImpulseManager {
    pub fn key_is_pressed(&mut self, callback: KeyIsPressed) {
        self.key_is_pressed.replace(callback);
    }
}
