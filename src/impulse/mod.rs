use std::fmt::Debug;

use winit::{dpi::PhysicalPosition, keyboard::KeyCode};

use crate::Isotope;

pub type KeyIsPressed = fn(KeyCode, &mut Isotope);
pub type KeyIsReleased = fn(KeyCode, &mut Isotope);
pub type CursorMoved = fn(PhysicalPosition<f64>, &mut Isotope);
pub type MouseIsMoved = fn((f64, f64), &mut Isotope);

#[derive(Debug, Default)]
pub struct ImpulseManager {
    pub(crate) key_is_pressed: Option<KeyIsPressed>,
    pub(crate) key_is_released: Option<KeyIsReleased>,
    pub(crate) cursor_moved: Option<CursorMoved>,
    pub(crate) mouse_is_moved: Option<MouseIsMoved>,
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

    pub fn cursor_moved(&mut self, callback: CursorMoved) -> &mut ImpulseManager {
        self.cursor_moved.replace(callback);
        self
    }

    pub fn mouse_is_moved(&mut self, callback: MouseIsMoved) -> &mut ImpulseManager {
        self.mouse_is_moved.replace(callback);
        self
    }
}
