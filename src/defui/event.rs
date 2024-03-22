use super::*;
use shell::MouseButton;

pub enum Event {
    /// being called every frame
    Update,
    MousePress(MouseButton),
    MouseUnpress(MouseButton),
}

pub enum Action {
    Quit,
    SetTitle(String),
    SetCursor(Cursor),
    SetBackgroundColor(Color),
}
