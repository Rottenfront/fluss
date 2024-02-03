use crate::*;
use std::{
    collections::{HashMap, VecDeque},
    sync::Mutex,
    time::{Duration, Instant},
};
use winit::{
    dpi::LogicalSize,
    event::{
        ElementState, Event as WEvent, MouseButton as WMouseButton, Touch, TouchPhase, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowBuilder},
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

fn process_event(cx: &mut Context, view: &impl View, event: &Event, window: &Window) {
    cx.process(view, event);

    if cx.grab_cursor && !cx.prev_grab_cursor {
        println!("grabbing cursor");
        window
            .set_cursor_grab(winit::window::CursorGrabMode::Locked)
            .or_else(|_e| window.set_cursor_grab(winit::window::CursorGrabMode::Confined))
            .unwrap();
        window.set_cursor_visible(false);
    }

    if !cx.grab_cursor && cx.prev_grab_cursor {
        println!("releasing cursor");
        window
            .set_cursor_grab(winit::window::CursorGrabMode::None)
            .unwrap();
        window.set_cursor_visible(true);
    }

    cx.prev_grab_cursor = cx.grab_cursor;
}

/// Call this function to run your UI.
pub fn rui(view: impl View) {
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let mut width = 800.0;
    let mut height = 800.0;
    let mut scale = 1.0;
    let mut window_title = String::from("rui");

    let winit_window_builder = WindowBuilder::new()
        .with_title("Fluss")
        .with_inner_size(LogicalSize::new(width, height))
        .with_transparent(true)
        .with_blur(true);

    let mut env = SkiaEnv::new(winit_window_builder, &event_loop);

    #[cfg(not(target_arch = "wasm32"))]
    {
        *GLOBAL_EVENT_LOOP_PROXY.lock().unwrap() = Some(event_loop.create_proxy());
    }

    let mut cx = Context::new();
    let mut mouse_position = LocalPoint::new(0.0, 0.0);

    let mut commands: Vec<CommandInfo> = Vec::new();
    let mut command_map = HashMap::new();
    cx.commands(&view, &mut commands);

    {
        // So we can infer a type for CommandMap when winit is enabled.
        command_map.insert("", "");
    }

    let mut access_nodes = vec![];

    // let font_mgr = FontMgr::new();
    // let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" height = "100" width = "100">
    //     <path d="M30,1h40l29,29v40l-29,29h-40l-29-29v-40z" stroke="#;000" fill="none"/>
    //     <path d="M31,3h38l28,28v38l-28,28h-38l-28-28v-38z" fill="#a23"/>
    //     <text x="50" y="68" font-size="48" fill="#FFF" text-anchor="middle"><![CDATA[410]]></text>
    //     </svg>"##;
    // let dom = SvgDom::from_str(svg, font_mgr).unwrap();
    let mut previous_frame_start = Instant::now();
    let mut modifiers = winit::event::Modifiers::default();
    let mut frame_duration = Duration::from_secs_f32(1.0 / 60.0);

    let state = env.get_drawer_state();
    let font = state
        .create_font(Font {
            name: "CaskaydiaCove Nerd Font".to_string(),
            size: 13.0,
            weight: Weight::Normal,
            width: Width::Normal,
        })
        .unwrap();

    event_loop
        .run(move |event, window_target| {
            let frame_start = Instant::now();
            let mut draw_frame = false;
            match event {
                WEvent::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        println!("The close button was pressed; stopping");
                        window_target.exit();
                        return;
                    }
                    WindowEvent::Resized(size) => {
                        width = size.width.max(1) as _;
                        height = size.height.max(1) as _;
                        env.on_resize(size);
                        env.request_redraw();
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
                                let event = Event::MousePress {
                                    id: 0,
                                    position: mouse_position,
                                };
                                process_event(&mut cx, &view, &event, env.window())
                            }
                            ElementState::Released => {
                                cx.mouse_button = None;
                                let event = Event::MouseUnpress {
                                    id: 0,
                                    position: mouse_position,
                                };
                                process_event(&mut cx, &view, &event, env.window())
                            }
                        };
                    }
                    WindowEvent::Touch(Touch {
                        phase, location, ..
                    }) => {
                        scale = env.window().scale_factor();
                        let position = [
                            location.x / scale as f64,
                            (height - location.y) / scale as f64,
                        ]
                        .into();

                        let delta = position - cx.previous_position[0];

                        // TODO: Multi-Touch management
                        let event = match phase {
                            TouchPhase::Started => Some(Event::MousePress { id: 0, position }),
                            TouchPhase::Moved => Some(Event::CursorMove {
                                id: 0,
                                position,
                                delta,
                            }),
                            TouchPhase::Ended | TouchPhase::Cancelled => {
                                Some(Event::MouseUnpress { id: 0, position })
                            }
                        };

                        if let Some(event) = event {
                            process_event(&mut cx, &view, &event, env.window());
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        scale = env.window().scale_factor();
                        mouse_position = (position.x / scale, (height - position.y) / scale).into();
                        // let event = Event::TouchMove {
                        //     id: 0,
                        //     position: mouse_position,
                        // };
                        // process_event(&mut cx, &view, &event, &window)
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        if event.state == ElementState::Pressed {
                            // TODO: handle keyboard input
                        }
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

                        draw_frame = true;
                    }
                    _ => {}
                },
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

                WEvent::DeviceEvent {
                    event: winit::event::DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    // Flip y coordinate.
                    let d: LocalOffset = (delta.0, -delta.1).into();

                    let event = Event::CursorMove {
                        id: 0,
                        position: mouse_position,
                        delta: d,
                    };

                    process_event(&mut cx, &view, &event, env.window());
                }
                _ => (),
            }
            if frame_start - previous_frame_start > frame_duration {
                draw_frame = true;
                previous_frame_start = frame_start;
            }
            let window_size = env.window().inner_size();
            scale = env.window().scale_factor();
            // println!("window_size: {:?}", window_size);
            width = window_size.width as f64 / scale;
            height = window_size.height as f64 / scale;
            let draw_state = env.get_drawer_state();

            if cx.update(&view, draw_state, &mut access_nodes, (width, height).into()) {
                env.request_redraw();
            }

            if cx.window_title != window_title {
                window_title = cx.window_title.clone();
                env.window().set_title(&cx.window_title);
            }

            if draw_frame {
                env.prepare_draw();
                let canvas = env.get_drawer();
                let mut drawer = SkiaDrawer::new(canvas.0, canvas.1);

                cx.render(&view, &mut drawer, (width, height).into(), scale);
                env.draw();
            }

            window_target.set_control_flow(ControlFlow::WaitUntil(
                previous_frame_start + frame_duration,
            ))
        })
        .expect("run() failed");
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
