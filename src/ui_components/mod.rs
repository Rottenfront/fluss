pub mod editor;
pub mod split;
pub mod tab_system;

use std::path::PathBuf;

use winit::{event::MouseButton, keyboard::Key};

pub enum WidgetEvent {
    CursorMove((f32, f32)),
    CursorLeft,
    ButtonPress(MouseButton),
    ButtonRelease(MouseButton),
    Scroll {
        delta: (f32, f32),
    },
    KeyboardInput {
        key: Key,
        shift: bool,
        logo: bool,
        ctrl: bool,
        alt: bool,
    },
    Disabled,
    Enabled,

    /// A file has been dropped into the widget.
    ///
    /// When the user drops multiple files at once, this event will be emitted for each file
    /// separately.
    DroppedFile(PathBuf),

    /// A file is being hovered over the widget.
    ///
    /// When the user hovers multiple files at once, this event will be emitted for each file
    /// separately.
    HoveredFile(PathBuf),

    /// A file was hovered, but has exited the widget.
    ///
    /// There will be a single `HoveredFileCancelled` event triggered even if multiple files were
    /// hovered.
    HoveredFileCancelled,
}

pub trait EventHandler {
    fn handle_widget_event(&mut self, _event: WidgetEvent) {}
}
