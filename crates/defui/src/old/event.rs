use std::path::PathBuf;

use crate::*;

/// Represents logical key name
#[derive(Clone, Debug)]
pub enum Key {
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Escape,
    Insert,
    PrintSc,
    Delete,
    Home,
    End,
    PgUp,
    PgDown,
    Tilda,
    Tab,
    Caps,
    LShift,
    RShift,
    LCtrl,
    RCtrl,
    /// Synonimes: Mod4, Logo, Win, Cmd
    Super,
    /// Synonimes: LOption
    LAlt,
    /// Synonimes: ROption
    RAlt,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    /// Synonimes: Enter
    Return,
    Backspace,
    Space,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Num0,
    Minus,
    Equal,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    BracketOpen,
    BracketClose,
    BraceOpen,
    BraceClose,
    Point,
    Comma,
    Semicolon,
    Quote,
    Slash,
    Backslash,
}

/// User interface event.
#[derive(Clone, Debug)]
pub enum Event {
    /// Touch event, or mouse down.
    MousePress {
        button: MouseButton,
        position: LocalPoint,
    },

    /// Touch moved or mouse moved while down.
    CursorMove {
        position: LocalPoint,
        delta: LocalOffset,
    },

    /// Touch went up or mouse button released.
    MouseUnpress {
        button: MouseButton,
        position: LocalPoint,
    },

    Key(Key, Option<char>),

    /// A file has been dropped into the window.
    ///
    /// When the user drops multiple files at once, this event will be emitted for each file
    /// separately.
    DroppedFile {
        file: PathBuf,
        position: LocalPoint,
    },

    /// A file is being hovered over the window.
    ///
    /// When the user hovers multiple files at once, this event will be emitted for each file
    /// separately.
    HoveredFile {
        file: PathBuf,
        position: LocalPoint,
    },

    /// A file was hovered, but has exited the window.
    ///
    /// There will be a single `HoveredFileCancelled` event triggered even if multiple files were
    /// hovered.
    HoveredFileCancelled,

    /// The window gained or lost focus.
    ///
    /// The parameter is true if the view has gained focus, and false if it has lost focus.
    Focused(bool),

    CursorEntered,

    CursorLeft,

    None,
}

impl Event {
    pub fn offset(&self, offset: LocalOffset) -> Event {
        let mut event = self.clone();
        match &mut event {
            Event::MousePress { position, .. } => *position += offset,
            Event::CursorMove { position, .. } => *position += offset,
            Event::MouseUnpress { position, .. } => *position += offset,
            _ => (),
        }
        event
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MouseButton {
    Left,
    Right,
    Center,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct KeyboardModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub command: bool,
}
