use std::path::PathBuf;

pub use kurbo::*;
use winit::event::{KeyEvent, MouseButton};

pub enum WidgetEvent {
    CursorMove((f32, f32)),
    CursorLeft,
    ButtonPress(MouseButton),
    ButtonRelease(MouseButton),
    Scroll {
        delta: (f32, f32),
    },
    KeyboardInput(KeyEvent),
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

// pub enum Primitive {
//     Rect {
//         top_left: Point,
//         bottom_right: Point,
//     },
//     RoundedRect {
//         top_left: Point,
//         bottom_right: Point,
//         radius: f32,
//     },
//     Text {
//         text: String,
//         top_left: Point,
//         font: FontId,
//     },
//     Path {
//         start: Point,
//         path: Vec<BezierCurve>,
//     },
// }

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

pub trait Context {
    fn create_image(&mut self, img: Image) -> ImageId;
    fn release_image(&mut self, id: ImageId) -> Result<(), String>;
}

pub trait Element {
    /// `self` field is not mutable 'cause it's better to use bindings for drawing context
    ///
    /// Binding doesn't require mutability to modify content
    fn draw(&self, max_bound: Point) -> Vec<(Box<impl Shape>, Filler)>;
    /// Returns true if event is handled, false if event is passed
    fn handle_event<Ctx: Context>(&mut self, event: WidgetEvent, ctx: &mut Ctx) -> bool;
    /// Must be called by context on widget creation
    fn prepare<Ctx: Context>(&mut self, ctx: &mut Ctx);
    /// Must be called by context on widget deletion, can be used for releasing used data
    fn delete<Ctx: Context>(&mut self, ctx: &mut Ctx);
}
