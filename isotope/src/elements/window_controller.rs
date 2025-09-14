use std::sync::Arc;

use log::warn;
pub use winit::window::CursorGrabMode;
use winit::window::Window;

pub struct WindowController {
    window: Arc<Window>,
    cursor_grab_mode: CursorGrabMode,
    cursor_visible: bool,
}

impl WindowController {
    pub(crate) fn new(window: Arc<Window>) -> Self {
        Self {
            window,
            cursor_grab_mode: CursorGrabMode::None,
            cursor_visible: true,
        }
    }

    fn update(&self) {
        self.window
            .set_cursor_grab(self.cursor_grab_mode)
            .unwrap_or_else(|err| {
                warn!(
                    "Failed to set cursor Grab mode with error: {}, continuing...",
                    err
                );
            });
        self.window.set_cursor_visible(self.cursor_visible);
    }

    /// Modifies the cursor grab mode for the window.
    ///
    /// The callback function receives a mutable reference to the current cursor grab mode,
    /// allowing you to modify it. After the callback executes, the window is automatically
    /// updated with the new settings.
    ///
    /// # Example
    /// ```
    /// window_controller.cursor_grab_mode(|mode| {
    ///     *mode = CursorGrabMode::Locked;
    /// });
    /// ```
    pub fn cursor_grab_mode<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut CursorGrabMode),
    {
        callback(&mut self.cursor_grab_mode);
        self.update();
    }

    /// Modifies the cursor visibility for the window.
    ///
    /// The callback function receives a mutable reference to the current cursor visibility state,
    /// allowing you to modify it. After the callback executes, the window is automatically
    /// updated with the new settings.
    ///
    /// # Example
    /// ```
    /// window_controller.cursor_visible(|visible| {
    ///     *visible = false;
    /// });
    /// ```
    pub fn cursor_visible<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut bool),
    {
        callback(&mut self.cursor_visible);
        self.update();
    }

    /// Modifies both the cursor grab mode and visibility for the window.
    ///
    /// The callback function receives mutable references to both the cursor grab mode and
    /// visibility state, allowing you to modify both simultaneously. After the callback
    /// executes, the window is automatically updated with the new settings.
    ///
    /// # Example
    /// ```
    /// window_controller.all(|grab_mode, visible| {
    ///     *grab_mode = CursorGrabMode::Locked;
    ///     *visible = false;
    /// });
    /// ```
    pub fn all<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut CursorGrabMode, &mut bool),
    {
        callback(&mut self.cursor_grab_mode, &mut self.cursor_visible);
        self.update();
    }
}
