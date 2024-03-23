mod app;
mod context;
mod event;
mod filler;
mod modifiers;
mod stack;
mod text;
mod view;

pub use context::*;
pub use event::*;
pub use filler::*;
pub use modifiers::*;
pub use stack::*;
pub use text::*;
pub use view::*;

pub use flo_binding::{bind, Binding, Bound, MutableBound};
pub use shell;

use shell::{piet::Color, Application, Cursor, WindowBuilder};
use std::any::Any;

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
    let uiapp = app::UIApp::new(ctx.set_root_view(view), ctx, window_properties);
    builder.set_handler(Box::new(uiapp));

    let window = builder.build().unwrap();

    window.show();

    app.run(None);
}
