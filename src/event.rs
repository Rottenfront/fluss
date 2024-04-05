use super::*;
pub use winit::{event::MouseButton, window::CursorIcon};

pub struct MouseUnpress {
    button: MouseButton,
    pos: Point,
}

pub struct MousePress {
    button: MouseButton,
    pos: Point,
}

pub struct ScrollEvent {
    pos: Point,
    delta: Vec2,
    sum_delta: Vec2,
}

pub enum Action {
    Quit,
    SetTitle(String),
    SetCursor(CursorIcon),
    SetBackgroundColor(Color),
}
