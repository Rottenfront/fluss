// use fluss::{defui::*, hstack, vstack, zstack};
//
// fn main() {
//     run(
//         hstack! {
//             zstack!{
//                 Filler::new(|| Color::RED),
//                 TextView::new(
//                     || "AAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA".into(),
//                     bind(Color::BLACK),
//                     bind(Font::MONOSPACE),
//                 )
//             },
//             zstack!{
//                 Filler::new(|| Color::GREEN),
//                 TextView::new(
//                     || "AAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA".into(),
//                     bind(Color::BLACK),
//                     bind(Font::MONOSPACE),
//                 )
//             },
//             vstack!{
//                 Clickable::new(Filler::new(|| Color::BLUE), |_, _, _| println!("press")),
//                 TextView::new(
//                     || "AAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA".into(),
//                     bind(Color::BLACK),
//                     bind(Font::MONOSPACE),
//                 )
//             }
//         },
//         WindowProperties {
//             title: "title".into(),
//             background_color: Color::GRAY,
//         },
//     )
// }

//! I PREFIX MEANS ICED STRUCTURE

use app::UIApp;
use winit::{
    event::{KeyEvent, WindowEvent},
    event_loop::EventLoop,
};

// struct Application {
//     event_loop: EventLoop<()>,
//     windows: Vec<AppWindow>,
// }

// impl AppWindow {
//     pub fn new<V: View + 'static>(root_view: V, window_properties: WindowProperties) -> Self {}
// }

mod app;
mod context;
mod event;
mod renderer;
mod view;
mod views;

pub use context::*;
pub use event::*;
pub use view::*;
pub use views::*;

pub use flo_binding::{bind, Binding, Bound, MutableBound};

pub use renderer::*;

pub struct WindowProperties {
    pub title: String,
    pub background_color: Color,
    pub transparent: bool,
    pub size: (u32, u32),
}

use winit::event::Event;

pub fn run<V: View + 'static>(view: V, window_properties: WindowProperties) {
    tracing_subscriber::fmt::init();

    // Initialize winit
    let event_loop = EventLoop::new().unwrap();

    let mut app = UIApp::new(&event_loop, view, window_properties);

    let mut resized = false;

    // Run event loop
    event_loop
        .run(move |event, window_target| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    if resized {
                        app.resize();
                        resized = false;
                    }

                    app.paint();
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        // cursor_position = Some(position);
                        app.mouse_move(&MouseMoveEvent {
                            pos: Point::new(position.x, position.y),
                        });
                        app.request_redraw();
                    }
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        // modifiers = new_modifiers.state();
                    }
                    WindowEvent::Resized(_) => {
                        resized = true;
                        app.request_redraw();
                    }
                    WindowEvent::CloseRequested => {
                        window_target.exit();
                    }
                    _ => {}
                },
                _ => {}
            }
        })
        .unwrap();
}
