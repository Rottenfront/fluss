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
use shell::{piet::Color, Application, Cursor, WindowBuilder};
pub use stack::*;
pub use text::*;
pub use view::*;

use std::any::Any;

pub use flo_binding::{bind, Binding, Bound, MutableBound};
pub use shell;

pub struct WindowProperties {
    pub title: String,
    pub backdround_color: Color,
}

pub fn run<V: View + 'static, F: Fn(&mut Context) -> V>(
    view: F,
    window_properties: WindowProperties,
) {
    tracing_subscriber::fmt().init();
    let app = Application::new().unwrap();
    let mut builder = WindowBuilder::new(app.clone());

    builder.set_title(&window_properties.title);
    // we set transparent to true so user can make window background transparent
    builder.set_transparent(true);

    let mut ctx = Context::new();
    let view = view(&mut ctx);
    let uiapp = app::UIApp::new(view, ctx, window_properties);
    builder.set_handler(Box::new(uiapp));

    let window = builder.build().unwrap();

    window.show();

    app.run(None);
}
