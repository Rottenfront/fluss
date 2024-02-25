use crate::*;

use std::time::{Duration, Instant};
use std::{
    collections::{HashMap, VecDeque},
    sync::Mutex,
};

use winit::{
    event::{
        ElementState, Event as WEvent, MouseButton as WMouseButton, Touch, TouchPhase, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::WindowBuilder,
};

type WorkQueue = VecDeque<Box<dyn FnOnce(&mut Context) + Send>>;

#[cfg(not(target_arch = "wasm32"))]
lazy_static! {
    /// Allows us to wake the event loop whenever we want.
    static ref GLOBAL_EVENT_LOOP_PROXY: Mutex<Option<EventLoopProxy<()>>> = Mutex::new(None);

    static ref GLOBAL_WORK_QUEUE: Mutex<WorkQueue> = Mutex::new(WorkQueue::new());
}

#[cfg(not(target_arch = "wasm32"))]
pub fn on_main(f: impl FnOnce(&mut Context) + Send + 'static) {
    GLOBAL_WORK_QUEUE.lock().unwrap().push_back(Box::new(f));

    // Wake up the event loop.
    let opt_proxy = GLOBAL_EVENT_LOOP_PROXY.lock().unwrap();
    if let Some(proxy) = &*opt_proxy {
        if let Err(err) = proxy.send_event(()) {
            println!("error waking up event loop: {:?}", err);
        }
    }
}

fn process_event(cx: &mut Context, view: &impl View, event: &Event, env: &mut DrawerEnv) {
    cx.process(view, event);

    if cx.grab_cursor && !cx.prev_grab_cursor {
        println!("grabbing cursor");
        env.window()
            .set_cursor_grab(winit::window::CursorGrabMode::Locked)
            .or_else(|_e| {
                env.window()
                    .set_cursor_grab(winit::window::CursorGrabMode::Confined)
            })
            .unwrap();
        env.window().set_cursor_visible(false);
    }

    if !cx.grab_cursor && cx.prev_grab_cursor {
        println!("releasing cursor");
        env.window()
            .set_cursor_grab(winit::window::CursorGrabMode::None)
            .unwrap();
        env.window().set_cursor_visible(true);
    }

    cx.prev_grab_cursor = cx.grab_cursor;
    env.window().request_redraw();
}

/// Call this function to run your UI.
pub fn rui(view: impl View) {
    static EXPECTED_FRAME_DURATION: f32 = 1.0 / 60.0;
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        let query_string = web_sys::window().unwrap().location().search().unwrap();
        let level: log::Level = parse_url_query_string(&query_string, "RUST_LOG")
            .map(|x| x.parse().ok())
            .flatten()
            .unwrap_or(log::Level::Error);
        console_log::init_with_level(level).expect("could not initialize logger");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
    }
    let event_loop = EventLoop::new().unwrap();

    let mut window_title = String::from("rui");
    let builder = WindowBuilder::new().with_title(&window_title);

    let mut env = DrawerEnv::new(builder, &event_loop);

    #[cfg(not(target_arch = "wasm32"))]
    {
        *GLOBAL_EVENT_LOOP_PROXY.lock().unwrap() = Some(event_loop.create_proxy());
    }

    let mut cx = Context::new();
    let mut mouse_position = Point::new(0.0, 0.0);
    let (mut width, mut height) = (
        env.window().inner_size().width as f64,
        env.window().inner_size().height as f64,
    );
    let mut scale = env.window().scale_factor();

    let mut commands: Vec<CommandInfo> = Vec::new();
    let mut command_map = HashMap::new();
    cx.commands(&view, &mut commands);

    {
        // So we can infer a type for CommandMap when winit is enabled.
        command_map.insert("", "");
    }

    // let mut access_nodes = vec![];

    let state = env.get_drawer_state();
    let font = state
        .create_font(Font {
            name: "CaskaydiaCove Nerd Font".to_string(),
            size: 13.0,
            weight: Weight::Normal,
            width: Width::Normal,
        })
        .unwrap();
    let mut previous_frame_start = Instant::now();
    let frame_duration = Duration::from_secs_f32(EXPECTED_FRAME_DURATION);
    event_loop
        .run(move |event, window_target| {
            let frame_start = Instant::now();
            let mut draw_frame = false;
            // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
            // dispatched any events. This is ideal for games and similar applications.
            // *control_flow = ControlFlow::Poll;

            // ControlFlow::Wait pauses the event loop if no events are available to process.
            // This is ideal for non-game applications that only update in response to user
            // input, and uses significantly less power/CPU time than ControlFlow::Poll.

            match event {
                WEvent::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    println!("The close button was pressed; stopping");
                    window_target.exit();
                    return;
                }
                WEvent::UserEvent(_) => {
                    // println!("received user event");

                    // Process the work queue.
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        while let Some(f) = GLOBAL_WORK_QUEUE.lock().unwrap().pop_front() {
                            f(&mut cx);
                        }
                    }
                }
                WEvent::WindowEvent {
                    window_id, event, ..
                } => match event {
                    WindowEvent::Resized(size) => {
                        // println!("Resizing to {:?}", size);
                        width = (size.width as f64).max(1.0);
                        height = (size.height as f64).max(1.0);
                        env.on_resize(size);
                        env.window().request_redraw();
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        match state {
                            ElementState::Pressed => {
                                cx.mouse_button = match button {
                                    WMouseButton::Left => Some(MouseButton::Left),
                                    WMouseButton::Right => Some(MouseButton::Right),
                                    WMouseButton::Middle => Some(MouseButton::Center),
                                    _ => None,
                                };
                                let event = Event::TouchBegin {
                                    id: 0,
                                    position: mouse_position,
                                };
                                process_event(&mut cx, &view, &event, &mut env)
                            }
                            ElementState::Released => {
                                cx.mouse_button = None;
                                let event = Event::TouchEnd {
                                    id: 0,
                                    position: mouse_position,
                                };
                                process_event(&mut cx, &view, &event, &mut env)
                            }
                        };
                    }
                    WindowEvent::Touch(Touch {
                        phase, location, ..
                    }) => {
                        // Do not handle events from other windows.
                        if window_id != env.window().id() {
                            return;
                        }

                        let scale = env.window().scale_factor() as f64;
                        let position = (
                            location.x as f64 / scale,
                            (height as f64 - location.y as f64) / scale,
                        )
                            .into();

                        let delta = position - cx.previous_position[0];

                        // TODO: Multi-Touch management
                        let event = match phase {
                            TouchPhase::Started => Some(Event::TouchBegin { id: 0, position }),
                            TouchPhase::Moved => Some(Event::TouchMove {
                                id: 0,
                                position,
                                delta,
                            }),
                            TouchPhase::Ended | TouchPhase::Cancelled => {
                                Some(Event::TouchEnd { id: 0, position })
                            }
                        };

                        if let Some(event) = event {
                            process_event(&mut cx, &view, &event, &mut env);
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let scale = env.window().scale_factor() as f64;
                        let new_mouse_position = (
                            position.x as f64 / scale,
                            (height as f64 - position.y as f64) / scale,
                        )
                            .into();
                        let event = Event::TouchMove {
                            id: 0,
                            position: new_mouse_position,
                            delta: new_mouse_position - mouse_position,
                        };
                        mouse_position = new_mouse_position;
                        process_event(&mut cx, &view, &event, &mut env);
                    }

                    WindowEvent::KeyboardInput { event, .. } => {
                        if event.state == ElementState::Pressed {}
                    }
                    WindowEvent::ModifiersChanged(mods) => {
                        cx.key_mods = KeyboardModifiers {
                            shift: mods.state().shift_key(),
                            control: mods.state().control_key(),
                            alt: mods.state().alt_key(),
                            command: mods.state().super_key(),
                        };
                    }
                    WindowEvent::RedrawRequested => {
                        // Redraw the application.
                        //
                        // It's preferable for applications that do not render continuously to render in
                        // this event rather than in MainEventsCleared, since rendering in here allows
                        // the program to gracefully handle redraws requested by the OS.

                        let window_size = env.window().inner_size();
                        scale = env.window().scale_factor() as f64;
                        // println!("window_size: {:?}", window_size);
                        width = window_size.width as f64 / scale;
                        height = window_size.height as f64 / scale;

                        // println!("RedrawRequested");
                        draw_frame = true;
                    }
                    _ => (),
                },

                WEvent::DeviceEvent {
                    event: winit::event::DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    // Flip y coordinate.
                    let d: Vec2 = (delta.0 as f64, -delta.1 as f64).into();

                    let event = Event::TouchMove {
                        id: 0,
                        position: mouse_position,
                        delta: d,
                    };

                    process_event(&mut cx, &view, &event, &mut env);
                }
                _ => (),
            };
            if frame_start - previous_frame_start > frame_duration {
                draw_frame = true;
                previous_frame_start = frame_start;
            }

            if draw_frame {
                env.prepare_draw();

                if cx.window_title != window_title {
                    window_title = cx.window_title.clone();
                    env.window().set_title(&cx.window_title);
                }
                cx.render(&view, &mut env, (width, height).into());
                env.draw();
            }

            window_target.set_control_flow(ControlFlow::WaitUntil(
                previous_frame_start + frame_duration,
            ))
        })
        .unwrap();
}

#[cfg(target_arch = "wasm32")]
/// Parse the query string as returned by `web_sys::window()?.location().search()?` and get a
/// specific key out of it.
pub fn parse_url_query_string<'a>(query: &'a str, search_key: &str) -> Option<&'a str> {
    let query_string = query.strip_prefix('?')?;

    for pair in query_string.split('&') {
        let mut pair = pair.split('=');
        let key = pair.next()?;
        let value = pair.next()?;

        if key == search_key {
            return Some(value);
        }
    }

    None
}

pub trait Run: View + Sized {
    fn run(self) {
        rui(self)
    }
}

impl<V: View> Run for V {}
