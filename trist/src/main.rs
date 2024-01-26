#[cfg(feature = "skia")]
mod skia_backend;

use std::time::{Duration, Instant};

#[cfg(feature = "skia")]
pub use skia_backend::{SkiaDrawer, SkiaEnv, SkiaDrawerState};

use kurbo::Shape;

use skia_safe::Canvas;
use winit::{dpi::{LogicalSize, PhysicalSize}, event::{Event, Modifiers, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::{Window, WindowBuilder}};

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
    r: f32,
    g: f32,
    b: f32,
    a: f32,
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
    name: String,
    size: f32,
    weight: Weight,
    width: Width,
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
    // clip shapes
    fn clip_shape(&mut self, shape: &impl Shape) -> Result<(), DrawerError>;
    fn unclip(&mut self);
    // draw shapes
    fn draw_shape(&mut self, shape: &impl Shape, paint: PaintId) -> Result<(), DrawerError>;
    fn draw_text(&mut self, text: String, x: f32, y: f32, font: FontId, paint: PaintId) -> Result<(), DrawerError>;
    fn state(&mut self) -> &mut T;
}

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

static EXPECTED_FRAME_DURATION: f32 = 1.0 / 60.0;
fn main() {
    let el = EventLoop::new().expect("Failed to create event loop");
    let winit_window_builder = WindowBuilder::new()
        .with_title("Fluss")
        .with_inner_size(LogicalSize::new(800, 800))
        .with_transparent(true)
        .with_blur(true);

    let mut env = SkiaEnv::new(winit_window_builder, &el);

    // let font_mgr = FontMgr::new();
    // let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" height = "100" width = "100">
    //     <path d="M30,1h40l29,29v40l-29,29h-40l-29-29v-40z" stroke="#;000" fill="none"/>
    //     <path d="M31,3h38l28,28v38l-28,28h-38l-28-28v-38z" fill="#a23"/>
    //     <text x="50" y="68" font-size="48" fill="#FFF" text-anchor="middle"><![CDATA[410]]></text>
    //     </svg>"##;
    // let dom = SvgDom::from_str(svg, font_mgr).unwrap();
    let mut previous_frame_start: Instant = Instant::now();
    let mut modifiers: Modifiers = Modifiers::default();
    let mut frame_duration: Duration = Duration::from_secs_f32(EXPECTED_FRAME_DURATION);

    let state = env.get_drawer_state();
    let font = state.create_font(Font { name: "CaskaydiaCove Nerd Font".to_string(), size: 13.0, weight: Weight::Normal, width: Width::Normal }).unwrap();

    el.run(move |event, window_target| {
        let frame_start = Instant::now();
        let mut draw_frame = false;
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::CloseRequested => {
                    window_target.exit();
                    return;
                }
                WindowEvent::Resized(physical_size) => {
                    env.on_resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    draw_frame = true;
                }
                _ => (),
            }
        }
        if frame_start - previous_frame_start > frame_duration {
            draw_frame = true;
            previous_frame_start = frame_start;
        }

        if draw_frame {
            env.prepare_draw();
            let canvas = env.get_drawer();
            let mut drawer = SkiaDrawer::new(canvas.0, canvas.1);
            let black = drawer.state().create_fast_paint(Paint::Color(Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 })).unwrap();
            let white = drawer.state().create_fast_paint(Paint::Color(Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 })).unwrap();
            drawer.draw_shape(&kurbo::Circle::new((100.0, 100.0), 100.0), black);
            drawer.draw_text("chlen".to_string(), 100.0, 100.0, font, white);
            env.draw();
        }

        window_target.set_control_flow(ControlFlow::WaitUntil(
            previous_frame_start + frame_duration,
        ))
    })
    .expect("run() failed");
}
