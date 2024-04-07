use super::*;
pub use winit::{event::MouseButton, window::CursorIcon};

pub struct MouseUnpress {
    pub button: MouseButton,
    pub pos: Point,
}

pub struct MousePress {
    pub button: MouseButton,
    pub pos: Point,
}

pub struct ScrollEvent {
    pub pos: Point,
    pub delta: Vec2,
    pub sum_delta: Vec2,
}

pub enum Action {
    Quit,
    SetTitle(String),
    SetCursor(CursorIcon),
    SetBackgroundColor(Color),
}
