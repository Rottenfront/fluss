use super::*;
use shell::{kurbo::Point, MouseButton};

pub enum Event {
    /// being called every frame
    Update,
    MousePress {
        button: MouseButton,
        pos: Point,
    },
    MouseUnpress {
        button: MouseButton,
        pos: Point,
    },
}

pub enum Action {
    Quit,
    SetTitle(String),
    SetCursor(Cursor),
    SetBackgroundColor(Color),
}
