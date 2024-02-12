use crate::*;
use std::{
    collections::{HashMap, VecDeque},
    sync::Mutex,
    time::{Duration, Instant},
};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event as WEvent, MouseButton as WMouseButton, WindowEvent},
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

fn handle_event(cx: &mut Context, view: &mut impl View, event: &Event, env: &mut DrawerEnv) {
    let draw_state = env.get_drawer_state();
    cx.handle_event(view, draw_state, event);

    let window = env.window();

    if cx.grab_cursor && !cx.prev_grab_cursor {
        debug!("grabbing cursor");
        window
            .set_cursor_grab(winit::window::CursorGrabMode::Locked)
            .or_else(|_e| window.set_cursor_grab(winit::window::CursorGrabMode::Confined))
            .unwrap();
        window.set_cursor_visible(false);
    }

    if !cx.grab_cursor && cx.prev_grab_cursor {
        debug!("releasing cursor");
        window
            .set_cursor_grab(winit::window::CursorGrabMode::None)
            .unwrap();
        window.set_cursor_visible(true);
    }

    cx.prev_grab_cursor = cx.grab_cursor;
}

fn setup_context_and_env() {}

/// Call this function to run your UI.
pub fn run_view_winit(view: impl View) {
    let mut view = view;
    let mut cx = Context::new();

    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let winit_window_builder = WindowBuilder::new()
        .with_title(cx.window_properties.window_title())
        .with_inner_size(LogicalSize::new(
            cx.window_properties.window_size.width,
            cx.window_properties.window_size.height,
        ))
        .with_transparent(true)
        .with_blur(true);

    let mut env = DrawerEnv::new(winit_window_builder, &event_loop);

    #[cfg(not(target_arch = "wasm32"))]
    {
        *GLOBAL_EVENT_LOOP_PROXY.lock().unwrap() = Some(event_loop.create_proxy());
    }

    let mut mouse_position = Point::new(0.0, 0.0);

    // let mut commands: Vec<CommandInfo> = Vec::new();
    let mut command_map = HashMap::new();
    // cx.commands(&view, &mut commands);

    {
        // So we can infer a type for CommandMap when winit is enabled.
        command_map.insert("", "");
    }

    // let mut access_nodes = vec![];

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
                        window_target.exit();
                        return;
                    }
                    WindowEvent::Resized(size) => {
                        env.on_resize(size);
                        env.request_redraw();
                    }

                    WindowEvent::MouseInput { state, button, .. } => 'mouse_input: {
                        let button = match button {
                            WMouseButton::Left => MouseButton::Left,
                            WMouseButton::Right => MouseButton::Right,
                            WMouseButton::Middle => MouseButton::Center,
                            _ => break 'mouse_input,
                        };
                        match state {
                            ElementState::Pressed => {
                                let event = Event::MousePress {
                                    button,
                                    position: mouse_position,
                                };
                                handle_event(&mut cx, &mut view, &event, &mut env);
                            }
                            ElementState::Released => {
                                let event = Event::MouseUnpress {
                                    button,
                                    position: mouse_position,
                                };
                                handle_event(&mut cx, &mut view, &event, &mut env);
                            }
                        };
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        cx.window_properties.scale = env.window().scale_factor();
                        mouse_position = (
                            position.x / cx.window_properties.scale,
                            (cx.window_properties.window_size.height - position.y)
                                / cx.window_properties.scale,
                        )
                            .into();
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
                    let d = (delta.0, -delta.1).into();

                    let event = Event::CursorMove {
                        position: mouse_position,
                        delta: d,
                    };

                    handle_event(&mut cx, &mut view, &event, &mut env);
                }
                _ => (),
            }
            if frame_start - previous_frame_start > frame_duration {
                draw_frame = true;
                previous_frame_start = frame_start;
            }

            if draw_frame {
                env.prepare_draw();
                let canvas = env.get_drawer();
                let mut drawer = Drawer::new(canvas.0, canvas.1);

                cx.render(&view, &mut drawer);
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
        run_view_winit(self)
    }
}

impl<V: View> Run for V {}
