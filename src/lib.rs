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

use std::time::{Duration, Instant};

use winit::{
    event::{Event as WEvent, KeyEvent, Modifiers, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use winit::dpi::PhysicalSize;

// struct AppWindow {
//     // root_view: Box<dyn View>,
//     renderer: Renderer,
//     window: Window,
//     frame_duration: Duration,
//     previous_frame_start: Instant,
//     modifiers: Modifiers,

//     properties: WindowProperties,
// }

// struct Application {
//     event_loop: EventLoop<()>,
//     windows: Vec<AppWindow>,
// }

// impl AppWindow {
//     pub fn new<V: View + 'static>(root_view: V, window_properties: WindowProperties) -> Self {}
// }

// mod app;
mod context;
mod event;
mod renderer;
mod view;
// mod views;

pub use context::*;
pub use event::*;
pub use view::*;
// pub use views::*;

pub use flo_binding::{bind, Binding, Bound, MutableBound};

pub use renderer::*;

pub struct WindowProperties {
    pub title: String,
    pub background_color: Color,
    pub transparent: bool,
    pub size: (u32, u32),
}

pub async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_blur(true)
        .with_transparent(true)
        .with_inner_size(PhysicalSize::new(1200, 800))
        .build(&event_loop)
        .unwrap();
    let mut renderer = Renderer::new(&window).await;

    // TODO: how to enable vsync?
    let expected_frame_length_seconds = 1.0 / 144.0;
    let frame_duration = Duration::from_secs_f32(expected_frame_length_seconds);

    let mut previous_frame_start = Instant::now();
    let mut modifiers = Modifiers::default();
    event_loop
        .run(|event, window_target| {
            let frame_start = Instant::now();
            let mut draw_frame = false;

            if let WEvent::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested => {
                        window_target.exit();
                        return;
                    }
                    WindowEvent::Resized(physical_size) => {
                        /* First resize the opengl drawable */
                        renderer.resize(physical_size)
                    }
                    WindowEvent::ModifiersChanged(new_modifiers) => modifiers = new_modifiers,
                    WindowEvent::KeyboardInput {
                        event: KeyEvent { logical_key, .. },
                        ..
                    } => {
                        if modifiers.state().super_key() && logical_key == "q" {
                            window_target.exit();
                        }
                        window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        draw_frame = true;
                    }
                    _ => (),
                }
            }
            let frame_time = frame_start - previous_frame_start;
            if frame_time > frame_duration {
                draw_frame = true;
                previous_frame_start = frame_start;
            }
            if draw_frame {
                renderer.fill_rect(
                    RectQuad::new((0.0, 0.0).into(), (100.0, 100.0).into())
                        .with_color(Color::BLACK)
                        .with_radius(10.0)
                        .with_border_width(10.0)
                        .with_border_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                );

                renderer.present();
            }

            window_target.set_control_flow(ControlFlow::WaitUntil(
                previous_frame_start + frame_duration,
            ))
        })
        .expect("Failed to run event loop");
}
