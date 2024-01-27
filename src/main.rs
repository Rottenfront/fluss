mod backend;
// mod public_api;
pub mod canvas;
mod ui_components;

use backend::{SkiaBackend, SkiaEnv};
use canvas::{Drawer, FontId};
use flo_binding::bind;
use skia_safe::{
    font_style::{Slant, Weight, Width},
    Canvas, Color, Font, FontMgr, FontStyle, Rect, Size,
};
use std::{
    collections::HashMap,
    path::Path,
    time::{Duration, Instant},
};
use ui_components::editor::*;
use ui_components::split::*;
use ui_components::tab_system::*;
use winit::{
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
    event::{
        ElementState, Event, Ime, KeyEvent, Modifiers, MouseButton, MouseScrollDelta, TouchPhase,
        WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    keyboard::PhysicalKey,
    window::WindowBuilder,
};

#[allow(dead_code)]
enum AppFocus {
    LeftDock,
    RightDock,
    BottomDock,
    Editors,
    None,
}

static EXPECTED_FRAME_DURATION: f64 = 1.0 / 60.0;

struct ApplicationState {
    editors: Split,
    focus: AppFocus,
    previous_frame_start: Instant,
    modifiers: Modifiers,
    frame_duration: Duration,
    keypress_map: HashMap<PhysicalKey, bool>,
    mouse_map: HashMap<MouseButton, bool>,
}

pub const UPPER_BAR_HEIGHT: f32 = 40.0;
pub const LEFT_BAR_WIDTH: f32 = 40.0;
pub const RIGHT_BAR_WIDTH: f32 = 40.0;
pub const BOTTOM_BAR_HEIGHT: f32 = 30.0;
pub const MONOSPACE_FONT_ID: FontId = FontId(0);
pub const SERIF_FONT_ID: FontId = FontId(1);

impl ApplicationState {
    pub fn init(env: &mut SkiaEnv) -> Self {
        env.window().set_ime_allowed(true);

        let text = std::fs::read_to_string(&Path::new("t.txt")).unwrap();
        Self {
            editors: Split {
                direction: SplitDirection::Horizontal,
                main_item: Box::new(SplitItem::TabSystem(TabSystem {
                    scroll: 0.0,
                    states: vec![EditorState {
                        file: text
                            .lines()
                            .map(|s| s.to_owned().chars().collect::<Vec<char>>())
                            .collect::<Vec<Vec<char>>>(),
                        name: "name".to_string(),
                        path: None,
                        scroll: (0.0, 0.0),
                        cursors: vec![Cursor {
                            position: (0, 0),
                            selection_pos: (0, 0),
                            normal_x: 0,
                            last_ime_len: 0,
                        }],
                        line_height: bind(14.0),
                        char_width: bind(2.0),
                        pointer_pos: bind((0.0, 0.0)),
                        lmb_pressed: bind(false),
                        rmb_pressed: bind(false),
                        editor_size: bind((0.0, 0.0)),
                        drawing_pos: bind((0.0, 0.0)),
                        cursor_instant: bind(Instant::now()),
                        tab_length: 4,
                        selection_rectangle_radius: 3.0,

                        braces: vec!['(', '[', '{', '"', '\''],
                        matching_braces: vec![')', ']', '}', '"', '\''],
                    }],
                    enabled: 0,
                })),
                next_item: None,
                fraction: 1.0,
                enabled: false,
            },
            focus: AppFocus::Editors,

            previous_frame_start: Instant::now(),
            modifiers: Modifiers::default(),
            frame_duration: Duration::from_secs_f64(EXPECTED_FRAME_DURATION),

            keypress_map: HashMap::new(),
            mouse_map: HashMap::new(),
        }
    }

    /// Renders a rectangle that occupies exactly half of the canvas
    pub fn draw(&self, canvas: &Canvas) {
        let canvas_size = Size::from(canvas.base_layer_size());

        canvas.clear(Color::WHITE);

        let drawer = Drawer {
            canvas,
            fonts: {
                let mut fonts = HashMap::new();
                fonts.insert(
                    MONOSPACE_FONT_ID,
                    Font::new(
                        FontMgr::new()
                            .match_family_style(
                                "CaskaydiaCove Nerd Font",
                                FontStyle::new(Weight::NORMAL, Width::NORMAL, Slant::Upright),
                            )
                            .unwrap(),
                        13.0,
                    ),
                );
                fonts
            },
        };

        self.editors.draw(
            &drawer,
            Rect::from_ltrb(
                LEFT_BAR_WIDTH,
                UPPER_BAR_HEIGHT,
                canvas_size.width - RIGHT_BAR_WIDTH,
                canvas_size.height - BOTTOM_BAR_HEIGHT,
            ),
        );
    }

    pub fn handle_event<T>(
        &mut self,
        event: Event<T>,
        window_target: &EventLoopWindowTarget<T>,
        env: &mut SkiaEnv,
    ) {
        let frame_start = Instant::now();
        let mut draw_frame = false;
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::CloseRequested => {
                    self.handle_close_request();
                    window_target.exit();
                    return;
                }
                WindowEvent::Resized(physical_size) => {
                    self.handle_window_resize(physical_size.clone(), env);
                    env.on_resize(physical_size);
                }
                WindowEvent::MouseWheel { delta, phase, .. } => {
                    self.handle_scroll(phase, delta, env)
                }
                WindowEvent::ModifiersChanged(new_modifiers) => self.modifiers = new_modifiers,
                WindowEvent::Ime(ime) => self.handle_ime_event(ime, env),
                WindowEvent::KeyboardInput { event, .. } => {
                    self.handle_keyboard_event(event, env, window_target);
                }
                WindowEvent::RedrawRequested => {
                    draw_frame = true;
                }
                WindowEvent::CursorMoved { position, .. } => {
                    self.handle_cursor_moved(position, env)
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    self.handle_mouse_input(state, button, env)
                }
                _ => (),
            }
        }
        if frame_start - self.previous_frame_start > self.frame_duration {
            draw_frame = true;
            self.previous_frame_start = frame_start;
        }
        if draw_frame {
            env.draw(|canvas| self.draw(canvas));
        }

        window_target.set_control_flow(ControlFlow::WaitUntil(
            self.previous_frame_start + self.frame_duration,
        ))
    }

    fn handle_window_resize(&mut self, size: PhysicalSize<u32>, env: &mut SkiaEnv) {}

    fn handle_keyboard_event<T>(
        &mut self,
        event: KeyEvent,
        env: &mut SkiaEnv,
        window_target: &EventLoopWindowTarget<T>,
    ) {
        let KeyEvent {
            physical_key,
            logical_key,
            repeat,
            ..
        } = event;
        if self.modifiers.state().super_key() && logical_key == "q" {
            window_target.exit();
        }
        if !self.keypress_map.contains_key(&physical_key) {
            self.keypress_map.insert(physical_key, false);
        }
        // check if key wasn't pressed before, because winit sends this event on key unpress too
        if !self.keypress_map[&physical_key] || repeat {
            match self.focus {
                AppFocus::Editors => {
                    for id in 0..self
                        .editors
                        .focused_tab_system()
                        .focused_editor()
                        .cursors
                        .len()
                    {
                        self.editors
                            .focused_tab_system_mut()
                            .focused_editor_mut()
                            .handle_cursor_input(id, logical_key.clone(), &self.modifiers);
                    }
                    self.editors
                        .focused_tab_system_mut()
                        .focused_editor_mut()
                        .sync_cursor_time();
                }
                _ => {}
            }
        }
        if !repeat {
            let b = self.keypress_map.remove(&physical_key).unwrap();
            self.keypress_map.insert(physical_key, !b);
        }
        env.request_redraw();
    }

    fn handle_ime_event(&mut self, ime: Ime, _env: &mut SkiaEnv) {
        for id in 0..self
            .editors
            .focused_tab_system_mut()
            .focused_editor_mut()
            .cursors
            .len()
        {
            self.editors
                .focused_tab_system_mut()
                .focused_editor_mut()
                .handle_ime(id, &ime)
        }
        self.editors
            .focused_tab_system_mut()
            .focused_editor_mut()
            .sync_cursor_time();
    }

    fn handle_mouse_input(
        &mut self,
        _state: ElementState,
        button: MouseButton,
        _env: &mut SkiaEnv,
    ) {
        if !self.mouse_map.contains_key(&button) {
            self.mouse_map.insert(button, false);
        }
        if !self.mouse_map[&button] {
            match button {
                MouseButton::Left => {
                    self.editors
                        .focused_tab_system_mut()
                        .focused_editor_mut()
                        .handle_left_mouse_press(&self.modifiers);
                }
                _ => {}
            }
        } else {
            match button {
                MouseButton::Left => {
                    self.editors
                        .focused_tab_system_mut()
                        .focused_editor_mut()
                        .handle_left_mouse_release(&self.modifiers);
                }
                _ => {}
            }
        }
        self.editors
            .focused_tab_system_mut()
            .focused_editor_mut()
            .sync_cursor_time();
        let b = self.mouse_map.remove(&button).unwrap();
        self.mouse_map.insert(button, !b);
    }

    fn handle_cursor_moved(&mut self, position: PhysicalPosition<f64>, _env: &mut SkiaEnv) {
        self.editors
            .focused_tab_system_mut()
            .focused_editor_mut()
            .handle_mouse_movement((position.x as _, position.y as _));
    }

    fn handle_scroll(&mut self, phase: TouchPhase, delta: MouseScrollDelta, _env: &mut SkiaEnv) {
        match phase {
            TouchPhase::Started => {}
            TouchPhase::Moved => {
                self.editors
                    .focused_tab_system_mut()
                    .focused_editor_mut()
                    .handle_scroll(match delta {
                        MouseScrollDelta::LineDelta(x, y) => (x, y),
                        MouseScrollDelta::PixelDelta(pos) => (pos.x as _, -pos.y as _),
                    });
                self.editors
                    .focused_tab_system_mut()
                    .focused_editor_mut()
                    .sync_cursor_time();
            }
            TouchPhase::Ended => {}
            _ => {}
        }
    }

    fn handle_close_request(&mut self) {
        // todo clean resources
    }
}

fn main() {
    let el = EventLoop::new().expect("Failed to create event loop");
    let winit_window_builder = WindowBuilder::new()
        .with_title("Fluss")
        .with_inner_size(LogicalSize::new(800, 800))
        .with_transparent(true)
        .with_blur(true);

    let mut env = SkiaEnv::new(winit_window_builder, &el);
    let mut app = ApplicationState::init(&mut env);

    // let font_mgr = FontMgr::new();
    // let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" height = "100" width = "100">
    //     <path d="M30,1h40l29,29v40l-29,29h-40l-29-29v-40z" stroke="#;000" fill="none"/>
    //     <path d="M31,3h38l28,28v38l-28,28h-38l-28-28v-38z" fill="#a23"/>
    //     <text x="50" y="68" font-size="48" fill="#FFF" text-anchor="middle"><![CDATA[410]]></text>
    //     </svg>"##;
    // let dom = SvgDom::from_str(svg, font_mgr).unwrap();

    el.run(move |event, window_target| {
        app.handle_event(event, window_target, &mut env);
    })
    .expect("run() failed");
}
