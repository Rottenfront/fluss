use crate::*;

/// Describes [input method](https://en.wikipedia.org/wiki/Input_method) events.
///
/// This is also called a "composition event".
///
/// Most keypresses using a latin-like keyboard layout simply generate a [`WindowEvent::KeyboardInput`].
/// However, one couldn't possibly have a key for every single unicode character that the user might want to type
/// - so the solution operating systems employ is to allow the user to type these using _a sequence of keypresses_ instead.
///
/// A prominent example of this is accents - many keyboard layouts allow you to first click the "accent key", and then
/// the character you want to apply the accent to. In this case, some platforms will generate the following event sequence:
/// ```ignore
/// // Press "`" key
/// Ime::Preedit("`", Some((0, 0)))
/// // Press "E" key
/// Ime::Preedit("", None) // Synthetic event generated by winit to clear preedit.
/// Ime::Commit("é")
/// ```
///
/// Additionally, certain input devices are configured to display a candidate box that allow the user to select the
/// desired character interactively. (To properly position this box, you must use [`AppAction::SetImeCursorArea`].)
///
/// An example of a keyboard layout which uses candidate boxes is pinyin. On a latin keyboard the following event
/// sequence could be obtained:
/// ```ignore
/// // Press "A" key
/// Ime::Preedit("a", Some((1, 1)))
/// // Press "B" key
/// Ime::Preedit("a b", Some((3, 3)))
/// // Press left arrow key
/// Ime::Preedit("a b", Some((1, 1)))
/// // Press space key
/// Ime::Preedit("啊b", Some((3, 3)))
/// // Press space key
/// Ime::Preedit("", None) // Synthetic event generated by winit to clear preedit.
/// Ime::Commit("啊不")
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Ime {
    /// Notifies when a new composing text should be set at the cursor position.
    ///
    /// The value represents a pair of the preedit string and the cursor begin position and end
    /// position. When it's `None`, the cursor should be hidden. When `String` is an empty string
    /// this indicates that preedit was cleared.
    ///
    /// The cursor position is byte-wise indexed.
    Preedit(String, Option<(usize, usize)>),

    /// Notifies when text should be inserted into the editor widget.
    ///
    /// Right before this event winit will send empty [`Self::Preedit`] event.
    Commit(String),
}

/// Represents logical key name
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Key {
    /// <kbd>`</kbd> on a US keyboard. This is also called a backtick or grave.
    /// This is the <kbd>半角</kbd>/<kbd>全角</kbd>/<kbd>漢字</kbd>
    /// (hankaku/zenkaku/kanji) key on Japanese keyboards
    Backquote,
    /// Used for both the US <kbd>\\</kbd> (on the 101-key layout) and also for the key
    /// located between the <kbd>"</kbd> and <kbd>Enter</kbd> keys on row C of the 102-,
    /// 104- and 106-key layouts.
    /// Labeled <kbd>#</kbd> on a UK (102) keyboard.
    Backslash,
    /// <kbd>[</kbd> on a US keyboard.
    BracketLeft,
    /// <kbd>]</kbd> on a US keyboard.
    BracketRight,
    /// <kbd>,</kbd> on a US keyboard.
    Comma,
    /// <kbd>0</kbd> on a US keyboard.
    Digit0,
    /// <kbd>1</kbd> on a US keyboard.
    Digit1,
    /// <kbd>2</kbd> on a US keyboard.
    Digit2,
    /// <kbd>3</kbd> on a US keyboard.
    Digit3,
    /// <kbd>4</kbd> on a US keyboard.
    Digit4,
    /// <kbd>5</kbd> on a US keyboard.
    Digit5,
    /// <kbd>6</kbd> on a US keyboard.
    Digit6,
    /// <kbd>7</kbd> on a US keyboard.
    Digit7,
    /// <kbd>8</kbd> on a US keyboard.
    Digit8,
    /// <kbd>9</kbd> on a US keyboard.
    Digit9,
    /// <kbd>=</kbd> on a US keyboard.
    Equal,
    /// <kbd>a</kbd> on a US keyboard.
    /// Labeled <kbd>q</kbd> on an AZERTY (e.g., French) keyboard.
    KeyA,
    /// <kbd>b</kbd> on a US keyboard.
    KeyB,
    /// <kbd>c</kbd> on a US keyboard.
    KeyC,
    /// <kbd>d</kbd> on a US keyboard.
    KeyD,
    /// <kbd>e</kbd> on a US keyboard.
    KeyE,
    /// <kbd>f</kbd> on a US keyboard.
    KeyF,
    /// <kbd>g</kbd> on a US keyboard.
    KeyG,
    /// <kbd>h</kbd> on a US keyboard.
    KeyH,
    /// <kbd>i</kbd> on a US keyboard.
    KeyI,
    /// <kbd>j</kbd> on a US keyboard.
    KeyJ,
    /// <kbd>k</kbd> on a US keyboard.
    KeyK,
    /// <kbd>l</kbd> on a US keyboard.
    KeyL,
    /// <kbd>m</kbd> on a US keyboard.
    KeyM,
    /// <kbd>n</kbd> on a US keyboard.
    KeyN,
    /// <kbd>o</kbd> on a US keyboard.
    KeyO,
    /// <kbd>p</kbd> on a US keyboard.
    KeyP,
    /// <kbd>q</kbd> on a US keyboard.
    /// Labeled <kbd>a</kbd> on an AZERTY (e.g., French) keyboard.
    KeyQ,
    /// <kbd>r</kbd> on a US keyboard.
    KeyR,
    /// <kbd>s</kbd> on a US keyboard.
    KeyS,
    /// <kbd>t</kbd> on a US keyboard.
    KeyT,
    /// <kbd>u</kbd> on a US keyboard.
    KeyU,
    /// <kbd>v</kbd> on a US keyboard.
    KeyV,
    /// <kbd>w</kbd> on a US keyboard.
    /// Labeled <kbd>z</kbd> on an AZERTY (e.g., French) keyboard.
    KeyW,
    /// <kbd>x</kbd> on a US keyboard.
    KeyX,
    /// <kbd>y</kbd> on a US keyboard.
    /// Labeled <kbd>z</kbd> on a QWERTZ (e.g., German) keyboard.
    KeyY,
    /// <kbd>z</kbd> on a US keyboard.
    /// Labeled <kbd>w</kbd> on an AZERTY (e.g., French) keyboard, and <kbd>y</kbd> on a
    /// QWERTZ (e.g., German) keyboard.
    KeyZ,
    /// <kbd>-</kbd> on a US keyboard.
    Minus,
    /// <kbd>.</kbd> on a US keyboard.
    Period,
    /// <kbd>'</kbd> on a US keyboard.
    Quote,
    /// <kbd>;</kbd> on a US keyboard.
    Semicolon,
    /// <kbd>/</kbd> on a US keyboard.
    Slash,
    /// <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
    AltLeft,
    /// <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
    /// This is labeled <kbd>AltGr</kbd> on many keyboard layouts.
    AltRight,
    /// <kbd>Backspace</kbd> or <kbd>⌫</kbd>.
    /// Labeled <kbd>Delete</kbd> on Apple keyboards.
    Backspace,
    /// <kbd>CapsLock</kbd> or <kbd>⇪</kbd>
    CapsLock,
    /// The application context menu key, which is typically found between the right
    /// <kbd>Super</kbd> key and the right <kbd>Control</kbd> key.
    ContextMenu,
    /// <kbd>Control</kbd> or <kbd>⌃</kbd>
    ControlLeft,
    /// <kbd>Control</kbd> or <kbd>⌃</kbd>
    ControlRight,
    /// <kbd>Enter</kbd> or <kbd>↵</kbd>. Labeled <kbd>Return</kbd> on Apple keyboards.
    Enter,
    /// The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
    SuperLeft,
    /// The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
    SuperRight,
    /// <kbd>Shift</kbd> or <kbd>⇧</kbd>
    ShiftLeft,
    /// <kbd>Shift</kbd> or <kbd>⇧</kbd>
    ShiftRight,
    /// <kbd> </kbd> (space)
    Space,
    /// <kbd>Tab</kbd> or <kbd>⇥</kbd>
    Tab,
    /// Japanese: <kbd>変</kbd> (henkan)
    Convert,
    /// <kbd>⌦</kbd>. The forward delete key.
    /// Note that on Apple keyboards, the key labelled <kbd>Delete</kbd> on the main part of
    /// the keyboard is encoded as [`Backspace`].
    ///
    /// [`Backspace`]: Self::Backspace
    Delete,
    /// <kbd>Page Down</kbd>, <kbd>End</kbd>, or <kbd>↘</kbd>
    End,
    /// <kbd>Help</kbd>. Not present on standard PC keyboards.
    Help,
    /// <kbd>Home</kbd> or <kbd>↖</kbd>
    Home,
    /// <kbd>Insert</kbd> or <kbd>Ins</kbd>. Not present on Apple keyboards.
    Insert,
    /// <kbd>Page Down</kbd>, <kbd>PgDn</kbd>, or <kbd>⇟</kbd>
    PageDown,
    /// <kbd>Page Up</kbd>, <kbd>PgUp</kbd>, or <kbd>⇞</kbd>
    PageUp,
    /// <kbd>↓</kbd>
    ArrowDown,
    /// <kbd>←</kbd>
    ArrowLeft,
    /// <kbd>→</kbd>
    ArrowRight,
    /// <kbd>↑</kbd>
    ArrowUp,
    /// On the Mac, this is used for the numpad <kbd>Clear</kbd> key.
    NumLock,
    /// <kbd>0 Ins</kbd> on a keyboard. <kbd>0</kbd> on a phone or remote control
    Numpad0,
    /// <kbd>1 End</kbd> on a keyboard. <kbd>1</kbd> or <kbd>1 QZ</kbd> on a phone or remote control
    Numpad1,
    /// <kbd>2 ↓</kbd> on a keyboard. <kbd>2 ABC</kbd> on a phone or remote control
    Numpad2,
    /// <kbd>3 PgDn</kbd> on a keyboard. <kbd>3 DEF</kbd> on a phone or remote control
    Numpad3,
    /// <kbd>4 ←</kbd> on a keyboard. <kbd>4 GHI</kbd> on a phone or remote control
    Numpad4,
    /// <kbd>5</kbd> on a keyboard. <kbd>5 JKL</kbd> on a phone or remote control
    Numpad5,
    /// <kbd>6 →</kbd> on a keyboard. <kbd>6 MNO</kbd> on a phone or remote control
    Numpad6,
    /// <kbd>7 Home</kbd> on a keyboard. <kbd>7 PQRS</kbd> or <kbd>7 PRS</kbd> on a phone
    /// or remote control
    Numpad7,
    /// <kbd>8 ↑</kbd> on a keyboard. <kbd>8 TUV</kbd> on a phone or remote control
    Numpad8,
    /// <kbd>9 PgUp</kbd> on a keyboard. <kbd>9 WXYZ</kbd> or <kbd>9 WXY</kbd> on a phone
    /// or remote control
    Numpad9,
    /// <kbd>+</kbd>
    NumpadAdd,
    /// Found on the Microsoft Natural Keyboard.
    NumpadBackspace,
    /// <kbd>C</kbd> or <kbd>A</kbd> (All Clear). Also for use with numpads that have a
    /// <kbd>Clear</kbd> key that is separate from the <kbd>NumLock</kbd> key. On the Mac, the
    /// numpad <kbd>Clear</kbd> key is encoded as [`NumLock`].
    ///
    /// [`NumLock`]: Self::NumLock
    NumpadClear,
    /// <kbd>C</kbd> (Clear Entry)
    NumpadClearEntry,
    /// <kbd>,</kbd> (thousands separator). For locales where the thousands separator
    /// is a "." (e.g., Brazil), this key may generate a <kbd>.</kbd>.
    NumpadComma,
    /// <kbd>. Del</kbd>. For locales where the decimal separator is "," (e.g.,
    /// Brazil), this key may generate a <kbd>,</kbd>.
    NumpadDecimal,
    /// <kbd>/</kbd>
    NumpadDivide,
    NumpadEnter,
    /// <kbd>=</kbd>
    NumpadEqual,
    /// <kbd>#</kbd> on a phone or remote control device. This key is typically found
    /// below the <kbd>9</kbd> key and to the right of the <kbd>0</kbd> key.
    NumpadHash,
    /// <kbd>M</kbd> Add current entry to the value stored in memory.
    NumpadMemoryAdd,
    /// <kbd>M</kbd> Clear the value stored in memory.
    NumpadMemoryClear,
    /// <kbd>M</kbd> Replace the current entry with the value stored in memory.
    NumpadMemoryRecall,
    /// <kbd>M</kbd> Replace the value stored in memory with the current entry.
    NumpadMemoryStore,
    /// <kbd>M</kbd> Subtract current entry from the value stored in memory.
    NumpadMemorySubtract,
    /// <kbd>*</kbd> on a keyboard. For use with numpads that provide mathematical
    /// operations (<kbd>+</kbd>, <kbd>-</kbd> <kbd>*</kbd> and <kbd>/</kbd>).
    ///
    /// Use `NumpadStar` for the <kbd>*</kbd> key on phones and remote controls.
    NumpadMultiply,
    /// <kbd>(</kbd> Found on the Microsoft Natural Keyboard.
    NumpadParenLeft,
    /// <kbd>)</kbd> Found on the Microsoft Natural Keyboard.
    NumpadParenRight,
    /// <kbd>*</kbd> on a phone or remote control device.
    ///
    /// This key is typically found below the <kbd>7</kbd> key and to the left of
    /// the <kbd>0</kbd> key.
    ///
    /// Use <kbd>"NumpadMultiply"</kbd> for the <kbd>*</kbd> key on
    /// numeric keypads.
    NumpadStar,
    /// <kbd>-</kbd>
    NumpadSubtract,
    /// <kbd>Esc</kbd> or <kbd>⎋</kbd>
    Escape,
    /// <kbd>Fn</kbd> This is typically a hardware key that does not generate a separate code.
    Fn,
    /// <kbd>FLock</kbd> or <kbd>FnLock</kbd>. Function Lock key. Found on the Microsoft
    /// Natural Keyboard.
    FnLock,
    /// <kbd>PrtScr SysRq</kbd> or <kbd>Print Screen</kbd>
    PrintScreen,
    /// <kbd>Scroll Lock</kbd>
    ScrollLock,
    /// <kbd>Pause Break</kbd>
    Pause,
    /// Some laptops place this key to the left of the <kbd>↑</kbd> key.
    ///
    /// This also the "back" button (triangle) on Android.
    BrowserBack,
    BrowserFavorites,
    /// Some laptops place this key to the right of the <kbd>↑</kbd> key.
    BrowserForward,
    /// The "home" button on Android.
    BrowserHome,
    BrowserRefresh,
    BrowserSearch,
    BrowserStop,
    MediaPlayPause,
    MediaSelect,
    MediaStop,
    MediaTrackNext,
    MediaTrackPrevious,
    /// This key is placed in the function section on some Apple keyboards, replacing the
    /// <kbd>Eject</kbd> key.
    Power,
    Sleep,
    AudioVolumeDown,
    AudioVolumeMute,
    AudioVolumeUp,
    WakeUp,
    Abort,
    Resume,
    Suspend,
    /// Found on Sun’s USB keyboard.
    Again,
    /// Found on Sun’s USB keyboard.
    Copy,
    /// Found on Sun’s USB keyboard.
    Cut,
    /// Found on Sun’s USB keyboard.
    Find,
    /// Found on Sun’s USB keyboard.
    Open,
    /// Found on Sun’s USB keyboard.
    Paste,
    /// Found on Sun’s USB keyboard.
    Props,
    /// Found on Sun’s USB keyboard.
    Select,
    /// Found on Sun’s USB keyboard.
    Undo,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F1,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F2,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F3,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F4,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F5,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F6,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F7,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F8,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F9,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F10,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F11,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F12,
}

impl Key {
    #[cfg(feature = "winit-backend")]
    pub fn from_winit_key(origin: winit::keyboard::KeyCode) -> Option<Self> {
        use winit::keyboard::KeyCode;
        match origin {
            KeyCode::Backquote => Some(Self::Backquote),
            KeyCode::Backslash => Some(Self::Backslash),
            KeyCode::BracketLeft => Some(Self::BracketLeft),
            KeyCode::BracketRight => Some(Self::BracketRight),
            KeyCode::Comma => Some(Self::Comma),
            KeyCode::Digit0 => Some(Self::Digit0),
            KeyCode::Digit1 => Some(Self::Digit1),
            KeyCode::Digit2 => Some(Self::Digit2),
            KeyCode::Digit3 => Some(Self::Digit3),
            KeyCode::Digit4 => Some(Self::Digit4),
            KeyCode::Digit5 => Some(Self::Digit5),
            KeyCode::Digit6 => Some(Self::Digit6),
            KeyCode::Digit7 => Some(Self::Digit7),
            KeyCode::Digit8 => Some(Self::Digit8),
            KeyCode::Digit9 => Some(Self::Digit9),
            KeyCode::Equal => Some(Self::Equal),
            KeyCode::IntlBackslash => Some(Self::Backslash),
            KeyCode::IntlRo => Some(Self::Backslash),
            KeyCode::IntlYen => Some(Self::Backslash),
            KeyCode::KeyA => Some(Self::KeyA),
            KeyCode::KeyB => Some(Self::KeyB),
            KeyCode::KeyC => Some(Self::KeyC),
            KeyCode::KeyD => Some(Self::KeyD),
            KeyCode::KeyE => Some(Self::KeyE),
            KeyCode::KeyF => Some(Self::KeyF),
            KeyCode::KeyG => Some(Self::KeyG),
            KeyCode::KeyH => Some(Self::KeyH),
            KeyCode::KeyI => Some(Self::KeyI),
            KeyCode::KeyJ => Some(Self::KeyJ),
            KeyCode::KeyK => Some(Self::KeyK),
            KeyCode::KeyL => Some(Self::KeyL),
            KeyCode::KeyM => Some(Self::KeyM),
            KeyCode::KeyN => Some(Self::KeyN),
            KeyCode::KeyO => Some(Self::KeyO),
            KeyCode::KeyP => Some(Self::KeyP),
            KeyCode::KeyQ => Some(Self::KeyQ),
            KeyCode::KeyR => Some(Self::KeyR),
            KeyCode::KeyS => Some(Self::KeyS),
            KeyCode::KeyT => Some(Self::KeyT),
            KeyCode::KeyU => Some(Self::KeyU),
            KeyCode::KeyV => Some(Self::KeyV),
            KeyCode::KeyW => Some(Self::KeyW),
            KeyCode::KeyX => Some(Self::KeyX),
            KeyCode::KeyY => Some(Self::KeyY),
            KeyCode::KeyZ => Some(Self::KeyZ),
            KeyCode::Minus => Some(Self::Minus),
            KeyCode::Period => Some(Self::Period),
            KeyCode::Quote => Some(Self::Quote),
            KeyCode::Semicolon => Some(Self::Semicolon),
            KeyCode::Slash => Some(Self::Slash),
            KeyCode::AltLeft => Some(Self::AltLeft),
            KeyCode::AltRight => Some(Self::AltRight),
            KeyCode::Backspace => Some(Self::Backspace),
            KeyCode::CapsLock => Some(Self::CapsLock),
            KeyCode::ContextMenu => Some(Self::ContextMenu),
            KeyCode::ControlLeft => Some(Self::ControlLeft),
            KeyCode::ControlRight => Some(Self::ControlRight),
            KeyCode::Enter => Some(Self::Enter),
            KeyCode::SuperLeft => Some(Self::SuperLeft),
            KeyCode::SuperRight => Some(Self::SuperRight),
            KeyCode::ShiftLeft => Some(Self::ShiftLeft),
            KeyCode::ShiftRight => Some(Self::ShiftRight),
            KeyCode::Space => Some(Self::Space),
            KeyCode::Tab => Some(Self::Tab),
            KeyCode::Convert => Some(Self::Convert),
            KeyCode::Delete => Some(Self::Delete),
            KeyCode::End => Some(Self::End),
            KeyCode::Help => Some(Self::Help),
            KeyCode::Home => Some(Self::Home),
            KeyCode::Insert => Some(Self::Insert),
            KeyCode::PageDown => Some(Self::PageDown),
            KeyCode::PageUp => Some(Self::PageUp),
            KeyCode::ArrowDown => Some(Self::ArrowDown),
            KeyCode::ArrowLeft => Some(Self::ArrowLeft),
            KeyCode::ArrowRight => Some(Self::ArrowRight),
            KeyCode::ArrowUp => Some(Self::ArrowUp),
            KeyCode::NumLock => Some(Self::NumLock),
            KeyCode::Numpad0 => Some(Self::Numpad0),
            KeyCode::Numpad1 => Some(Self::Numpad1),
            KeyCode::Numpad2 => Some(Self::Numpad2),
            KeyCode::Numpad3 => Some(Self::Numpad3),
            KeyCode::Numpad4 => Some(Self::Numpad4),
            KeyCode::Numpad5 => Some(Self::Numpad5),
            KeyCode::Numpad6 => Some(Self::Numpad6),
            KeyCode::Numpad7 => Some(Self::Numpad7),
            KeyCode::Numpad8 => Some(Self::Numpad8),
            KeyCode::Numpad9 => Some(Self::Numpad9),
            KeyCode::NumpadAdd => Some(Self::NumpadAdd),
            KeyCode::NumpadBackspace => Some(Self::NumpadBackspace),
            KeyCode::NumpadClear => Some(Self::NumpadClear),
            KeyCode::NumpadClearEntry => Some(Self::NumpadClearEntry),
            KeyCode::NumpadComma => Some(Self::NumpadComma),
            KeyCode::NumpadDecimal => Some(Self::NumpadDecimal),
            KeyCode::NumpadDivide => Some(Self::NumpadDivide),
            KeyCode::NumpadEnter => Some(Self::NumpadEnter),
            KeyCode::NumpadEqual => Some(Self::NumpadEqual),
            KeyCode::NumpadHash => Some(Self::NumpadHash),
            KeyCode::NumpadMemoryAdd => Some(Self::NumpadMemoryAdd),
            KeyCode::NumpadMemoryClear => Some(Self::NumpadMemoryClear),
            KeyCode::NumpadMemoryRecall => Some(Self::NumpadMemoryRecall),
            KeyCode::NumpadMemoryStore => Some(Self::NumpadMemoryStore),
            KeyCode::NumpadMemorySubtract => Some(Self::NumpadMemorySubtract),
            KeyCode::NumpadMultiply => Some(Self::NumpadMultiply),
            KeyCode::NumpadParenLeft => Some(Self::NumpadParenLeft),
            KeyCode::NumpadParenRight => Some(Self::NumpadParenRight),
            KeyCode::NumpadStar => Some(Self::NumpadStar),
            KeyCode::NumpadSubtract => Some(Self::NumpadSubtract),
            KeyCode::Escape => Some(Self::Escape),
            KeyCode::Fn => Some(Self::Fn),
            KeyCode::FnLock => Some(Self::FnLock),
            KeyCode::PrintScreen => Some(Self::PrintScreen),
            KeyCode::ScrollLock => Some(Self::ScrollLock),
            KeyCode::Pause => Some(Self::Pause),
            KeyCode::BrowserBack => Some(Self::BrowserBack),
            KeyCode::BrowserFavorites => Some(Self::BrowserFavorites),
            KeyCode::BrowserForward => Some(Self::BrowserForward),
            KeyCode::BrowserHome => Some(Self::BrowserHome),
            KeyCode::BrowserRefresh => Some(Self::BrowserRefresh),
            KeyCode::BrowserSearch => Some(Self::BrowserSearch),
            KeyCode::BrowserStop => Some(Self::BrowserStop),
            KeyCode::MediaPlayPause => Some(Self::MediaPlayPause),
            KeyCode::MediaSelect => Some(Self::MediaSelect),
            KeyCode::MediaStop => Some(Self::MediaStop),
            KeyCode::MediaTrackNext => Some(Self::MediaTrackNext),
            KeyCode::MediaTrackPrevious => Some(Self::MediaTrackPrevious),
            KeyCode::Power => Some(Self::Power),
            KeyCode::Sleep => Some(Self::Sleep),
            KeyCode::AudioVolumeDown => Some(Self::AudioVolumeDown),
            KeyCode::AudioVolumeMute => Some(Self::AudioVolumeMute),
            KeyCode::AudioVolumeUp => Some(Self::AudioVolumeUp),
            KeyCode::WakeUp => Some(Self::WakeUp),
            KeyCode::Abort => Some(Self::Abort),
            KeyCode::Resume => Some(Self::Resume),
            KeyCode::Suspend => Some(Self::Suspend),
            KeyCode::Again => Some(Self::Again),
            KeyCode::Copy => Some(Self::Copy),
            KeyCode::Cut => Some(Self::Cut),
            KeyCode::Find => Some(Self::Find),
            KeyCode::Open => Some(Self::Open),
            KeyCode::Paste => Some(Self::Paste),
            KeyCode::Props => Some(Self::Props),
            KeyCode::Select => Some(Self::Select),
            KeyCode::Undo => Some(Self::Undo),
            KeyCode::F1 => Some(Self::F1),
            KeyCode::F2 => Some(Self::F2),
            KeyCode::F3 => Some(Self::F3),
            KeyCode::F4 => Some(Self::F4),
            KeyCode::F5 => Some(Self::F5),
            KeyCode::F6 => Some(Self::F6),
            KeyCode::F7 => Some(Self::F7),
            KeyCode::F8 => Some(Self::F8),
            KeyCode::F9 => Some(Self::F9),
            KeyCode::F10 => Some(Self::F10),
            KeyCode::F11 => Some(Self::F11),
            KeyCode::F12 => Some(Self::F12),
            _ => None,
        }
    }
}

/// User interface event.
#[derive(Clone, Debug)]
pub enum Event {
    /// Touch event, or mouse down.
    MousePress {
        button: MouseButton,
        position: Point,
    },

    /// Touch moved or mouse moved while down.
    CursorMove {
        position: Point,
        delta: Vec2,
    },

    /// Touch went up or mouse button released.
    MouseUnpress {
        button: MouseButton,
        position: Point,
    },

    KeyPress(Key, Option<char>),
    KeyUnpress(Key, Option<char>),

    Ime(Ime),

    /// The view gained or lost focus.
    ///
    /// The parameter is true if the view has gained focus, and false if it has lost focus.
    Focused(bool),

    CursorEntered,

    CursorLeft,

    None,
}

#[derive(Copy, Clone, Debug)]
pub enum MouseButton {
    Left,
    Right,
    Center,
    Other(usize),
}

#[derive(Copy, Clone, Debug, Default)]
pub struct KeyboardModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub command: bool,
}

impl KeyboardModifiers {
    pub fn new() -> Self {
        Self {
            shift: false,
            control: false,
            alt: false,
            command: false,
        }
    }
}