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

#[macro_export]
macro_rules! zstack {
    // The pattern for a single `eval`
    {$($view:expr),+} => {
        {
            let mut views = vec![];
            $(
                views.push(Box::new($view) as Box<(dyn View + 'static)>);
            )+
            Stack::zstack(views)
        }
    };
}

#[macro_export]
macro_rules! hstack {
    // The pattern for a single `eval`
    {$($view:expr),+} => {
        {
            let mut views = vec![];
            $(
                views.push(Box::new($view) as Box<(dyn View + 'static)>);
            )+
            Stack::hstack(views)
        }
    };
}

#[macro_export]
macro_rules! vstack {
    // The pattern for a single `eval`
    {$($view:expr),+} => {
        {
            let mut views = vec![];
            $(
                views.push(Box::new($view) as Box<(dyn View + 'static)>);
            )+
            Stack::vstack(views)
        }
    };
}

pub fn run<V: View + 'static>(view: V, window_properties: WindowProperties) {
    tracing_subscriber::fmt().init();
    let app = Application::new().unwrap();
    let mut builder = WindowBuilder::new(app.clone());

    builder.set_title(&window_properties.title);
    // we set transparent to true so user can make window background transparent
    builder.set_transparent(true);

    let uiapp = app::UIApp::new(view, window_properties);
    builder.set_handler(Box::new(uiapp));

    let window = builder.build().unwrap();

    window.show();

    app.run(None);
}
