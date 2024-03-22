mod app;
mod context;
mod event;
mod filler;
mod stack;
mod text;
mod view;

pub use context::*;
pub use event::*;
pub use filler::*;
use shell::{Application, Cursor, WindowBuilder};
pub use stack::*;
pub use text::*;
pub use view::*;

use std::any::Any;

pub use flo_binding::{bind, Binding, Bound, MutableBound};
pub use shell;

pub struct WindowProperties {
    pub title: Binding<String>,
    pub transparent: Binding<bool>,
    pub cursor_visible: Binding<bool>,
    pub cursor_grabbed: Binding<bool>,
    pub cursor: Binding<Cursor>,
}

pub fn run<V: View + 'static, F: Fn(&mut Context) -> V>(view: F, title: String) {
    tracing_subscriber::fmt().init();
    let app = Application::new().unwrap();
    let mut builder = WindowBuilder::new(app.clone());

    builder.set_title(&title);
    builder.set_transparent(true);

    let mut ctx = Context::new();
    let view = view(&mut ctx);
    let uiapp = app::UIApp::new(view, ctx, title);
    builder.set_handler(Box::new(uiapp));

    let window = builder.build().unwrap();

    window.show();

    app.run(None);
}
