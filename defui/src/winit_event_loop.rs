use crate::*;

use futures::executor::block_on;
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use winit::{
    dpi::PhysicalSize,
    event::{
        ElementState, Event as WEvent, MouseButton as WMouseButton, Touch, TouchPhase,
        VirtualKeyCode, WindowEvent,
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

    use trist::*;
    let event_loop = EventLoop::new();

    let mut window_title = String::from("rui");
    let builder = WindowBuilder::new().with_title(&window_title);
    let window = builder.build(&event_loop).unwrap();

    let setup = block_on(setup(&window));
    let surface = setup.surface;
    let device = Arc::new(setup.device);
    let size = setup.size;
    let adapter = setup.adapter;
    let queue = Arc::new(setup.queue);

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_capabilities(&adapter).formats[0],
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
    };
    surface.configure(&device, &config);

    #[cfg(not(target_arch = "wasm32"))]
    {
        *GLOBAL_EVENT_LOOP_PROXY.lock().unwrap() = Some(event_loop.create_proxy());
    }

    let mut vger = Vger::new(device.clone(), queue.clone(), config.format);
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
    let font = state.create_font(Font { name: "CaskaydiaCove Nerd Font".to_string(), size: 13.0, weight: Weight::Normal, width: Width::Normal }).unwrap();

    el.run(move |event, window_target| {
        let frame_start = Instant::now();
        let mut draw_frame = false;
        match event {
            WEvent::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                *control_flow = ControlFlow::Exit
            }
            WEvent::WindowEvent {
                event:
                    WindowEvent::Resized(size)
                    | WindowEvent::ScaleFactorChanged {
                        new_inner_size: &mut size,
                        ..
                    },
                ..
            } => {
                // println!("Resizing to {:?}", size);
                config.width = size.width.max(1);
                config.height = size.height.max(1);
                surface.configure(&device, &config);
                window.request_redraw();
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

                // println!("RedrawRequested");
                cx.render(
                    RenderInfo {
                        device: &device,
                        surface: &surface,
                        config: &config,
                        queue: &queue,
                    },
                    &view,
                    &mut vger,
                    [width, height].into(),
                    scale,
                );
            }
            WEvent::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
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
            WEvent::WindowEvent {
                window_id,
                event:
                    WindowEvent::Touch(Touch {
                        phase, location, ..
                    }),
                ..
            } => {
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
            WEvent::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
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

            WEvent::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if input.state == ElementState::Pressed {
                    if let Some(code) = input.virtual_keycode {
                        let key = match code {
                            // VirtualKeyCode::Character(c) => Some(Key::Character(c)),
                            VirtualKeyCode::Key1 => {
                                Some(Key::Character(if cx.key_mods.shift { '!' } else { '1' }))
                            }
                            VirtualKeyCode::Key2 => {
                                Some(Key::Character(if cx.key_mods.shift { '@' } else { '2' }))
                            }
                            VirtualKeyCode::Key3 => {
                                Some(Key::Character(if cx.key_mods.shift { '#' } else { '3' }))
                            }
                            VirtualKeyCode::Key4 => {
                                Some(Key::Character(if cx.key_mods.shift { '$' } else { '4' }))
                            }
                            VirtualKeyCode::Key5 => {
                                Some(Key::Character(if cx.key_mods.shift { '%' } else { '5' }))
                            }
                            VirtualKeyCode::Key6 => {
                                Some(Key::Character(if cx.key_mods.shift { '^' } else { '6' }))
                            }
                            VirtualKeyCode::Key7 => {
                                Some(Key::Character(if cx.key_mods.shift { '&' } else { '7' }))
                            }
                            VirtualKeyCode::Key8 => {
                                Some(Key::Character(if cx.key_mods.shift { '*' } else { '8' }))
                            }
                            VirtualKeyCode::Key9 => {
                                Some(Key::Character(if cx.key_mods.shift { '(' } else { '9' }))
                            }
                            VirtualKeyCode::Key0 => {
                                Some(Key::Character(if cx.key_mods.shift { ')' } else { '0' }))
                            }
                            VirtualKeyCode::A => {
                                Some(Key::Character(if cx.key_mods.shift { 'A' } else { 'a' }))
                            }
                            VirtualKeyCode::B => {
                                Some(Key::Character(if cx.key_mods.shift { 'B' } else { 'b' }))
                            }
                            VirtualKeyCode::C => {
                                Some(Key::Character(if cx.key_mods.shift { 'C' } else { 'c' }))
                            }
                            VirtualKeyCode::D => {
                                Some(Key::Character(if cx.key_mods.shift { 'D' } else { 'd' }))
                            }
                            VirtualKeyCode::E => {
                                Some(Key::Character(if cx.key_mods.shift { 'E' } else { 'e' }))
                            }
                            VirtualKeyCode::F => {
                                Some(Key::Character(if cx.key_mods.shift { 'F' } else { 'f' }))
                            }
                            VirtualKeyCode::G => {
                                Some(Key::Character(if cx.key_mods.shift { 'G' } else { 'g' }))
                            }
                            VirtualKeyCode::H => {
                                Some(Key::Character(if cx.key_mods.shift { 'H' } else { 'h' }))
                            }
                            VirtualKeyCode::I => {
                                Some(Key::Character(if cx.key_mods.shift { 'I' } else { 'i' }))
                            }
                            VirtualKeyCode::J => {
                                Some(Key::Character(if cx.key_mods.shift { 'J' } else { 'j' }))
                            }
                            VirtualKeyCode::K => {
                                Some(Key::Character(if cx.key_mods.shift { 'K' } else { 'k' }))
                            }
                            VirtualKeyCode::L => {
                                Some(Key::Character(if cx.key_mods.shift { 'L' } else { 'l' }))
                            }
                            VirtualKeyCode::M => {
                                Some(Key::Character(if cx.key_mods.shift { 'M' } else { 'm' }))
                            }
                            VirtualKeyCode::N => {
                                Some(Key::Character(if cx.key_mods.shift { 'N' } else { 'n' }))
                            }
                            VirtualKeyCode::O => {
                                Some(Key::Character(if cx.key_mods.shift { 'O' } else { 'o' }))
                            }
                            VirtualKeyCode::P => {
                                Some(Key::Character(if cx.key_mods.shift { 'P' } else { 'p' }))
                            }
                            VirtualKeyCode::Q => {
                                Some(Key::Character(if cx.key_mods.shift { 'Q' } else { 'q' }))
                            }
                            VirtualKeyCode::R => {
                                Some(Key::Character(if cx.key_mods.shift { 'R' } else { 'r' }))
                            }
                            VirtualKeyCode::S => {
                                Some(Key::Character(if cx.key_mods.shift { 'S' } else { 's' }))
                            }
                            VirtualKeyCode::T => {
                                Some(Key::Character(if cx.key_mods.shift { 'T' } else { 't' }))
                            }
                            VirtualKeyCode::U => {
                                Some(Key::Character(if cx.key_mods.shift { 'U' } else { 'u' }))
                            }
                            VirtualKeyCode::V => {
                                Some(Key::Character(if cx.key_mods.shift { 'V' } else { 'v' }))
                            }
                            VirtualKeyCode::W => {
                                Some(Key::Character(if cx.key_mods.shift { 'W' } else { 'w' }))
                            }
                            VirtualKeyCode::X => {
                                Some(Key::Character(if cx.key_mods.shift { 'X' } else { 'x' }))
                            }
                            VirtualKeyCode::Y => {
                                Some(Key::Character(if cx.key_mods.shift { 'Y' } else { 'y' }))
                            }
                            VirtualKeyCode::Z => {
                                Some(Key::Character(if cx.key_mods.shift { 'Z' } else { 'z' }))
                            }
                            VirtualKeyCode::Semicolon => {
                                Some(Key::Character(if cx.key_mods.shift { ':' } else { ';' }))
                            }
                            VirtualKeyCode::Colon => Some(Key::Character(':')),
                            VirtualKeyCode::Caret => Some(Key::Character('^')),
                            VirtualKeyCode::Asterisk => Some(Key::Character('*')),
                            VirtualKeyCode::Period => {
                                Some(Key::Character(if cx.key_mods.shift { '>' } else { '.' }))
                            }
                            VirtualKeyCode::Comma => {
                                Some(Key::Character(if cx.key_mods.shift { '<' } else { ',' }))
                            }
                            VirtualKeyCode::Equals | VirtualKeyCode::NumpadEquals => {
                                Some(Key::Character('='))
                            }
                            VirtualKeyCode::Plus | VirtualKeyCode::NumpadAdd => {
                                Some(Key::Character('+'))
                            }
                            VirtualKeyCode::Minus | VirtualKeyCode::NumpadSubtract => {
                                Some(Key::Character(if cx.key_mods.shift { '_' } else { '-' }))
                            }
                            VirtualKeyCode::Slash | VirtualKeyCode::NumpadDivide => {
                                Some(Key::Character(if cx.key_mods.shift { '?' } else { '/' }))
                            }
                            VirtualKeyCode::Grave => {
                                Some(Key::Character(if cx.key_mods.shift { '~' } else { '`' }))
                            }
                            VirtualKeyCode::Return => Some(Key::Enter),
                            VirtualKeyCode::Tab => Some(Key::Tab),
                            VirtualKeyCode::Space => Some(Key::Space),
                            VirtualKeyCode::Down => Some(Key::ArrowDown),
                            VirtualKeyCode::Left => Some(Key::ArrowLeft),
                            VirtualKeyCode::Right => Some(Key::ArrowRight),
                            VirtualKeyCode::Up => Some(Key::ArrowUp),
                            VirtualKeyCode::End => Some(Key::End),
                            VirtualKeyCode::Home => Some(Key::Home),
                            VirtualKeyCode::PageDown => Some(Key::PageDown),
                            VirtualKeyCode::PageUp => Some(Key::PageUp),
                            VirtualKeyCode::Back => Some(Key::Backspace),
                            VirtualKeyCode::Delete => Some(Key::Delete),
                            VirtualKeyCode::Escape => Some(Key::Escape),
                            VirtualKeyCode::F1 => Some(Key::F1),
                            VirtualKeyCode::F2 => Some(Key::F2),
                            VirtualKeyCode::F3 => Some(Key::F3),
                            VirtualKeyCode::F4 => Some(Key::F4),
                            VirtualKeyCode::F5 => Some(Key::F5),
                            VirtualKeyCode::F6 => Some(Key::F6),
                            VirtualKeyCode::F7 => Some(Key::F7),
                            VirtualKeyCode::F8 => Some(Key::F8),
                            VirtualKeyCode::F9 => Some(Key::F9),
                            VirtualKeyCode::F10 => Some(Key::F10),
                            VirtualKeyCode::F11 => Some(Key::F11),
                            VirtualKeyCode::F12 => Some(Key::F12),
                            _ => None,
                        };

                        if let Some(key) = key {
                            cx.process(&view, &Event::Key(key))
                        }
                    }
                }
            }

            WEvent::WindowEvent {
                event: WindowEvent::ModifiersChanged(mods),
                ..
            } => {
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
            _ => (),
        }
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::CloseRequested => {
                    window_target.exit();
                    return;
                }
                WindowEvent::Resized(physical_size) => {
                    env.on_resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    draw_frame = true;
                }
                _ => (),
            }
        }
        if frame_start - previous_frame_start > frame_duration {
            draw_frame = true;
            previous_frame_start = frame_start;
        }

        if draw_frame {
            env.prepare_draw();
            let canvas = env.get_drawer();
            let mut drawer = SkiaDrawer::new(canvas.0, canvas.1);
            
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
