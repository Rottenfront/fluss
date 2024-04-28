use std::time::{Duration, Instant};

use app::UIApp;
use winit::{
    event::{KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

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

    let mut previous_frame_start = Instant::now();
    let mut cursor_pos = Point::new(0.0, 0.0);

    // Run event loop
    event_loop
        .run(move |event, window_target| {
            let frame_start = Instant::now();

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        // cursor_position = Some(position);
                        cursor_pos = Point::new(position.x, position.y);
                        app.cursor_move(MouseMoveEvent { pos: cursor_pos });
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        if state.is_pressed() {
                            app.mouse_press(MousePress {
                                button,
                                pos: cursor_pos,
                            });
                        } else {
                            app.mouse_unpress(MouseUnpress {
                                button,
                                pos: cursor_pos,
                            });
                        }
                    }
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        // modifiers = new_modifiers.state();
                    }
                    WindowEvent::Resized(_) => {
                        app.resize();
                        app.request_redraw();
                    }
                    WindowEvent::CloseRequested => {
                        window_target.exit();
                    }
                    WindowEvent::RedrawRequested => {
                        app.paint();
                    }
                    _ => {}
                },
                _ => {}
            }

            let expected_frame_length_seconds = 1.0 / 60.0;
            let frame_duration = Duration::from_secs_f32(expected_frame_length_seconds);

            if frame_start - previous_frame_start > frame_duration {
                app.update();
                previous_frame_start = frame_start;
            }

            window_target.set_control_flow(ControlFlow::WaitUntil(
                previous_frame_start + frame_duration,
            ))
        })
        .unwrap();
}
