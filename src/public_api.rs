use std::path::PathBuf;

pub use kurbo::*;
use winit::{
    event::{KeyEvent, MouseButton},
    keyboard::Key,
};

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
    Resized((f32, f32)),
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

pub struct KeyboardInput {
    key: Key,
    shift: bool,
    logo: bool,
    ctrl: bool,
    alt: bool,
}

pub type FontId = usize;
pub const MONOSPACE_FONT: FontId = 0;
pub const SERIF_FONT: FontId = 1;

pub enum ImageFormat {
    Rgba,
}
pub struct Image {
    data: Vec<u8>,
    format: ImageFormat,
}
pub type ImageId = usize;

pub enum BezierCurve {
    Linear(Point),
    Quad(Point, Point),
    Cubic(Point, Point, Point),
}

pub trait BezierPathTrait {
    fn line_to(&mut self, point: Point);
    fn quad_to(&mut self, point1: Point, point2: Point);
    fn cubic_to(&mut self, point1: Point, point2: Point, point3: Point);
}

impl BezierPathTrait for Vec<BezierCurve> {
    fn line_to(&mut self, point: Point) {
        self.push(BezierCurve::Linear(point));
    }
    fn quad_to(&mut self, point1: Point, point2: Point) {
        self.push(BezierCurve::Quad(point1, point2));
    }
    fn cubic_to(&mut self, point1: Point, point2: Point, point3: Point) {
        self.push(BezierCurve::Cubic(point1, point2, point3));
    }
}

pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

pub enum Filler {
    Image(ImageId),
    Color(Color),
    LinearGradient((Point, Color), (Point, Color)),
}

pub trait WidgetContext {
    fn create_image(&mut self, img: Image) -> ImageId;
    fn delete_image(&mut self, id: ImageId) -> Option<Image>;
    fn require_focus(&mut self, focus: bool);
    fn transmit_focus(&mut self, next: bool);
}

pub enum Primitive {
    Rect(Rect),
    RoundedRect(RoundedRect),
}

pub trait Widget {
    /// `self` field is not mutable 'cause it's better to use bindings for drawing context
    ///
    /// Binding doesn't require mutability to modify content
    fn draw(&self, max_bound: Point) -> (Vec<(Box<impl Shape>, Filler)>, Rect);

    /// Handles only relative to widget coords
    ///
    /// Returns true if event is handled, false if event is passed
    fn handle_cursor_movement(&mut self, position: Point, ctx: &mut Box<dyn WidgetContext>)
        -> bool;
    /// Returns true if event is handled, false if event is passed
    fn handle_cursor_left(&mut self, ctx: &mut Box<dyn WidgetContext>) -> bool;
    /// Returns true if event is handled, false if event is passed
    fn handle_mouse_button_pressed(
        &mut self,
        button: &MouseButton,
        ctx: &mut Box<dyn WidgetContext>,
    ) -> bool;
    /// Returns true if event is handled, false if event is passed
    fn handle_mouse_button_released(
        &mut self,
        button: &MouseButton,
        ctx: &mut Box<dyn WidgetContext>,
    ) -> bool;
    /// Returns true if event is handled, false if event is passed
    fn handle_scroll(&mut self, delta: Vec2, ctx: &mut Box<dyn WidgetContext>) -> bool;
    /// Returns true if event is handled, false if event is passed
    fn handle_resized(&mut self, rect: Size, ctx: &mut Box<dyn WidgetContext>) -> bool;
    /// Returns true if event is handled, false if event is passed
    fn handle_disabled(&mut self, ctx: &mut Box<dyn WidgetContext>) -> bool;
    /// Returns true if event is handled, false if event is passed
    fn handle_enabled(&mut self, ctx: &mut Box<dyn WidgetContext>) -> bool;
    /// Returns true if event is handled, false if event is passed
    fn handle_keyboard_input(
        &mut self,
        event: KeyboardInput,
        ctx: &mut Box<dyn WidgetContext>,
    ) -> bool;
    /// Returns true if event is handled, false if event is passed
    fn handle_dropped_file(&mut self, path: PathBuf, ctx: &mut Box<dyn WidgetContext>) -> bool;
    /// Returns true if event is handled, false if event is passed
    fn handle_file_hover(&mut self, path: PathBuf, ctx: &mut Box<dyn WidgetContext>) -> bool;
    /// Returns true if event is handled, false if event is passed
    fn handle_file_hover_canceled(&mut self, ctx: &mut Box<dyn WidgetContext>) -> bool;

    /// Must be called by context on widget creation
    fn prepare(&mut self, ctx: &mut Box<dyn WidgetContext>);
    /// Must be called by context on widget deletion, can be used for releasing used data
    fn delete(&mut self, ctx: &mut Box<dyn WidgetContext>);
}
