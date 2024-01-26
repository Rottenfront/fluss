use crate::*;
use futures::executor::block_on;
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};
use trist::*;

use winit::{
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
    let event_loop = EventLoop::new();

    let mut window_title = String::from("rui");
    let builder = WindowBuilder::new().with_title(&window_title);

    let env = SkiaEnv::new(builder, &event_loop);

    #[cfg(not(target_arch = "wasm32"))]
    {
        *GLOBAL_EVENT_LOOP_PROXY.lock().unwrap() = Some(event_loop.create_proxy());
    }

    let mut cx = Context::new();
    let mut mouse_position = LocalPoint::zero();

    let mut commands: Vec<CommandInfo> = Vec::new();
    let mut command_map = HashMap::new();
    cx.commands(&view, &mut commands);

    {
        // So we can infer a type for CommandMap when winit is enabled.
        command_map.insert("", "");
    }

    let mut access_nodes = vec![];

    let el = EventLoop::new().expect("Failed to create event loop");
    let winit_window_builder = WindowBuilder::new()
        .with_title("Fluss")
        .with_inner_size(LogicalSize::new(800, 800))
        .with_transparent(true)
        .with_blur(true);

    let mut env = SkiaEnv::new(winit_window_builder, &el);

    // let font_mgr = FontMgr::new();
    // let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" height = "100" width = "100">
    //     <path d="M30,1h40l29,29v40l-29,29h-40l-29-29v-40z" stroke="#;000" fill="none"/>
    //     <path d="M31,3h38l28,28v38l-28,28h-38l-28-28v-38z" fill="#a23"/>
    //     <text x="50" y="68" font-size="48" fill="#FFF" text-anchor="middle"><![CDATA[410]]></text>
    //     </svg>"##;
    // let dom = SvgDom::from_str(svg, font_mgr).unwrap();
    let mut previous_frame_start: Instant = Instant::now();
    let mut modifiers: Modifiers = Modifiers::default();
    let mut frame_duration: Duration = Duration::from_secs_f32(EXPECTED_FRAME_DURATION);

    let state = env.get_drawer_state();
    let font = state
        .create_font(Font {
            name: "CaskaydiaCove Nerd Font".to_string(),
            size: 13.0,
            weight: Weight::Normal,
            width: Width::Normal,
        })
        .unwrap();

    el.run(move |event, window_target| {
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
                    config.width = size.width.max(1);
                    config.height = size.height.max(1);
                    env.on_resize(size);
                    window.request_redraw();
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
                            process_event(&mut cx, &view, &event, &window)
                        }
                        ElementState::Released => {
                            cx.mouse_button = None;
                            let event = Event::TouchEnd {
                                id: 0,
                                position: mouse_position,
                            };
                            process_event(&mut cx, &view, &event, &window)
                        }
                    };
                }
                WindowEvent::Touch(Touch {
                    phase, location, ..
                }) => {
                    // Do not handle events from other windows.
                    if window_id != window.id() {
                        return;
                    }

                    let scale = window.scale_factor() as f32;
                    let position = [
                        location.x as f32 / scale,
                        (config.height as f32 - location.y as f32) / scale,
                    ]
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
                        process_event(&mut cx, &view, &event, &window);
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let scale = window.scale_factor() as f32;
                    mouse_position = [
                        position.x as f32 / scale,
                        (config.height as f32 - position.y as f32) / scale,
                    ]
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
                        shift: mods.shift(),
                        control: mods.ctrl(),
                        alt: mods.alt(),
                        command: mods.logo(),
                    };
                }

                WEvent::DeviceEvent {
                    event: winit::event::DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    // Flip y coordinate.
                    let d: LocalOffset = [delta.0 as f32, -delta.1 as f32].into();

                    let event = Event::TouchMove {
                        id: 0,
                        position: mouse_position,
                        delta: d,
                    };

                    process_event(&mut cx, &view, &event, &window);
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
            WEvent::MainEventsCleared => {
                // Application update code.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw, in
                // applications which do not always need to. Applications that redraw continuously
                // can just render here instead.

                let window_size = window.inner_size();
                let scale = window.scale_factor() as f32;
                // println!("window_size: {:?}", window_size);
                let width = window_size.width as f32 / scale;
                let height = window_size.height as f32 / scale;

                if cx.update(&view, &mut vger, &mut access_nodes, [width, height].into()) {
                    window.request_redraw();
                }

                if cx.window_title != window_title {
                    window_title = cx.window_title.clone();
                    window.set_title(&cx.window_title);
                }
            }
            WEvent::RedrawRequested(_) => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in MainEventsCleared, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                let window_size = window.inner_size();
                let scale = window.scale_factor() as f32;
                // println!("window_size: {:?}", window_size);
                let width = window_size.width as f32 / scale;
                let height = window_size.height as f32 / scale;
                draw_frame = true;
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
            let mut drawer = SkiaDrawer::new(canvas.0, canvas.1);

            cx.render(
                RenderInfo {
                    device: &device,
                    surface: &surface,
                    config: &config,
                    queue: &queue,
                },
                &view,
                &mut drawer,
                [width, height].into(),
                scale,
            );
            env.draw();
        }

        window_target.set_control_flow(ControlFlow::WaitUntil(
            previous_frame_start + frame_duration,
        ))
    })
    .expect("run() failed");
    event_loop.run(move |event, _, control_flow| {
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events. This is ideal for games and similar applications.
        // *control_flow = ControlFlow::Poll;

        // ControlFlow::Wait pauses the event loop if no events are available to process.
        // This is ideal for non-game applications that only update in response to user
        // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        *control_flow = ControlFlow::Wait;
    });
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
