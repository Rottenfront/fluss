#[cfg(feature = "skia")]
mod skia_backend;

use std::time::{Duration, Instant};

#[cfg(feature = "skia")]
pub use skia_backend::{SkiaDrawer, SkiaDrawerState, SkiaEnv};

use kurbo::{Point, Shape, Size};

use skia_safe::Canvas;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{Event, Modifiers, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Copy, Debug)]
pub struct PaintId {
    id: usize,
    is_static: bool,
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Copy, Debug)]
pub struct FontId {
    id: usize,
}

#[derive(Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Debug)]
pub enum Paint {
    Color(Color),
}

#[derive(Debug)]
pub enum Width {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

#[derive(Debug)]
pub enum Weight {
    Invisible,
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    Semibold,
    Bold,
    ExtraBold,
    Black,
    ExtraBlack,
}

#[derive(Debug)]
pub struct Font {
    pub name: String,
    pub size: f32,
    pub weight: Weight,
    pub width: Width,
}

#[derive(Debug)]
pub enum DrawerError {
    NoPaint(PaintId),
    NoFont(FontId),
    CannotDraw,
    CannotCreatePaint(Paint),
    CannotCreateFont(Font),
}

pub trait DrawerState {
    // manage paints
    fn create_fast_paint(&mut self, paint: Paint) -> Result<PaintId, DrawerError>;
    fn create_static_paint(&mut self, paint: Paint) -> Result<PaintId, DrawerError>;
    fn remove_static_paint(&mut self, paint: PaintId) -> Result<(), DrawerError>;
    fn create_font(&mut self, font: Font) -> Result<FontId, DrawerError>;
    fn remove_font(&mut self, font: FontId) -> Result<(), DrawerError>;
}

pub trait Drawer<T: DrawerState> {
    fn save(&mut self) -> Result<(), DrawerError>;
    fn restore(&mut self) -> Result<(), DrawerError>;

    fn translate(&mut self, point: Point) -> Result<(), DrawerError>;

    fn clip_shape(&mut self, shape: &impl Shape) -> Result<(), DrawerError>;
    fn draw_shape(&mut self, shape: &impl Shape, paint: PaintId) -> Result<(), DrawerError>;
    fn draw_text(
        &mut self,
        text: String,
        x: f32,
        y: f32,
        font: FontId,
        paint: PaintId,
    ) -> Result<(), DrawerError>;
    fn state(&mut self) -> &mut T;
}

#[cfg(feature = "skia")]
pub trait TristBackend<S: DrawerState> {
    fn new<E>(window: winit::window::WindowBuilder, event_loop: &EventLoop<E>) -> Self;
    fn on_resize(&mut self, size: PhysicalSize<u32>);
    fn request_redraw(&mut self);
    fn prepare_draw(&mut self);
    fn get_drawer_state(&mut self) -> &mut S;
    fn get_drawer(&mut self) -> (&Canvas, &mut SkiaDrawerState);
    fn draw(&mut self);
    fn window(&mut self) -> &mut Window;
}
