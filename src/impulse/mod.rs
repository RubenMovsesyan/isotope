use std::fmt::Debug;

use winit::keyboard::KeyCode;

use crate::Isotope;

pub type KeyIsPressed = fn(KeyCode, &mut Isotope);
pub type KeyIsReleased = fn(KeyCode, &mut Isotope);

#[derive(Debug, Default)]
pub struct ImpulseManager {
    pub(crate) key_is_pressed: Option<KeyIsPressed>,
    pub(crate) key_is_released: Option<KeyIsReleased>,
}

impl ImpulseManager {
    pub fn key_is_pressed(&mut self, callback: KeyIsPressed) -> &mut ImpulseManager {
        self.key_is_pressed.replace(callback);
        self
    }

    pub fn key_is_released(&mut self, callback: KeyIsReleased) -> &mut ImpulseManager {
        self.key_is_released.replace(callback);
        self
    }
}
