use super::*;
use shell::MouseButton;

pub enum Event {
    /// being called every frame
    Update,
    MousePress(MouseButton),
    MouseUnpress(MouseButton),
}
