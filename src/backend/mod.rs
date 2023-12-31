#[cfg(feature = "gl-render")]
mod gl_backend;

#[cfg(feature = "gl-render")]
pub use gl_backend::SkiaEnv;

use skia_safe::Canvas;
use winit::{dpi::PhysicalSize, event_loop::EventLoop};

pub trait SkiaBackend {
    fn new<T>(window: winit::window::WindowBuilder, event_loop: &EventLoop<T>) -> Self;
    fn on_resize(&mut self, size: PhysicalSize<u32>);
    fn request_redraw(&mut self);
    fn draw(&mut self);
    fn canvas(&mut self) -> &Canvas;
}
