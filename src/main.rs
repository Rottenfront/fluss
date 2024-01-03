/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use futures::{executor, prelude::*};
use std::time::Duration;
use std::{cmp::min, path::PathBuf, sync::Arc, thread};

use flo_draw::*;
use flo_draw::{
    binding::{bind, BindRef, Binding, Bound, MutableBound},
    canvas::*,
    winit::{
        event::{MouseButton, MouseScrollDelta, TouchPhase},
        window::{CursorIcon, Theme, WindowLevel},
    },
};

#[derive(Debug, Clone)]
enum SpanType {
    Text,
    Comment,
    Keyword,
}

#[derive(Debug, Clone)]
struct EditorState {
    file: Vec<String>,
    name: String,
    path: Option<PathBuf>,
    scroll: (f32, f32),
    spans: Vec<(usize, usize, usize, usize, SpanType)>,
    cursors: Vec<(usize, usize, usize, usize)>,
}

#[derive(Debug, Clone)]
struct FSItem {
    name: String,
    is_folded: bool,
    level: usize,
}

#[derive(Debug, Clone)]
struct FSState {
    root: PathBuf,
    items: Vec<FSItem>,
}

#[derive(Debug, Clone)]
enum LeftItem {
    None,
    FS,
}

#[derive(Debug, Clone)]
struct LeftBar {
    enabled: LeftItem,
    // fs: FSState,
}

#[derive(Debug, Clone)]
enum RightItem {
    None,
}

#[derive(Debug, Clone)]
struct RightBar {
    enabled: RightItem,
}

#[derive(Debug, Clone)]
enum BottomItem {
    None,
    Terminal,
}

#[derive(Debug, Clone)]
struct TermState {}

#[derive(Debug, Clone)]
struct BottomBar {
    enabled: BottomItem,
    // terminal: TermState,
}

#[derive(Debug, Clone)]
struct InputState {
    pointer_pos: Binding<(f32, f32)>,
    pointer_key: Binding<Option<MouseButton>>,
    is_scroll_started: Binding<bool>,
    prev_scroll_delta: Binding<(f32, f32)>,
    scroll_delta: Binding<(f32, f32)>,
}

#[derive(Debug, Clone)]
struct State {
    size: Binding<(u64, u64)>,
    min_size: Binding<Option<(u64, u64)>>,
    max_size: Binding<Option<(u64, u64)>>,
    title: Binding<String>,
    is_transparent: Binding<bool>,
    is_visible: Binding<bool>,
    is_resizable: Binding<bool>,
    is_minimized: Binding<bool>,
    is_maximized: Binding<bool>,
    fullscreen: Binding<bool>,
    has_decorations: Binding<bool>,
    window_level: Binding<WindowLevel>,
    ime_position: Binding<(u64, u64)>,
    ime_allowed: Binding<bool>,
    theme: Binding<Option<Theme>>,
    cursor_position: Binding<(u64, u64)>,
    cursor_icon: Binding<MousePointer>,
    is_exit_emmited: Binding<bool>,
    is_needed_redraw: Binding<bool>,

    input_state: InputState,

    editors: Vec<EditorState>,
    enabled_editor: Option<usize>,

    left_bound: f32,
    right_bound: f32,
    bottom_bound: f32,

    left_items: LeftBar,
    right_items: RightBar,
    bottom_items: BottomBar,

    serif_font: Arc<CanvasFontFace>,
    serif_font_size: f32,
    monospace_font: Arc<CanvasFontFace>,
    monospace_font_size: f32,
}

const SERIF_FONT: FontId = FontId(1);
const MONOSPACE_FONT: FontId = FontId(2);

const UPPER_BAR_HEIGHT: f32 = 40.0;
const LEFT_BAR_WIDTH: f32 = 40.0;
const RIGHT_BAR_WIDTH: f32 = 40.0;
const BOTTOM_BAR_HEIGHT: f32 = 30.0;

struct ColorTheme {
    background: Color,
    text: Color,
    inactive_text: Color,
    selection: Color,

    sidebar_back: Color,
    sidebar_button_back: Color,
    sidebar_button_hover: Color,
    sidebar_button_active: Color,
}

impl EditorState {
    fn draw(&self, gc: &mut Vec<Draw>, x1: f32, y1: f32, x2: f32, y2: f32, metrics: &FontMetrics) {
        let first_line = (self.scroll.1 / metrics.height) as usize;
        let last_line = ((self.scroll.1 + (y2 - y1)) / metrics.height) as usize + 1;
        // println!("{first_line}, {last_line}");
        // gc.unclip();
        gc.new_path();
        gc.rect(x1, y1, x2, y2);
        gc.clip();

        let delta_y = self.scroll.1 - metrics.height * (first_line as f32);
        let y2 = y2 + delta_y - metrics.height;
        gc.fill_color(Color::Rgba(0.0, 0.0, 0.0, 1.0));
        for i in first_line..min(last_line, self.file.len()) {
            gc.begin_line_layout(
                x1,
                y2 - (i - first_line) as f32 * metrics.height,
                TextAlignment::Left,
            );
            gc.layout_text(MONOSPACE_FONT, self.file[i].clone());
            gc.draw_text_layout();
            // gc.fill_text(self.file[i].as_str(), x1, y2, metrics.em_size, Color::Rgba(0.0, 0.0, 0.0, 1.0));
        }

        gc.unclip();
    }
}

impl State {
    fn draw(&self, gc: &mut Vec<Draw>) {
        let (width, height) = self.size.get();
        let width = width as _;
        let height = height as _;
        // Clear the canvas and set up the coordinates
        gc.clear_canvas(Color::Rgba(1.0, 1.0, 1.0, 1.0));
        gc.canvas_height(height);
        gc.center_region(0.0, 0.0, width, height);
        // Load the fonts
        gc.define_font_data(SERIF_FONT, Arc::clone(&self.serif_font));
        gc.define_font_data(MONOSPACE_FONT, Arc::clone(&self.monospace_font));
        gc.set_font_size(SERIF_FONT, self.serif_font_size);
        gc.set_font_size(MONOSPACE_FONT, self.monospace_font_size);
        self.draw_editor(gc);
        let (x, y) = self.input_state.pointer_pos.get();
        // Draw on layer 1 to avoid disrupting the image underneath
        gc.layer(LayerId(3));
        gc.clear_layer();

        gc.new_path();
        gc.circle(x as _, y as _, 20.0);

        gc.stroke_color(Color::Rgba(0.1, 0.1, 0.1, 0.8));
        gc.line_width_pixels(3.0);
        gc.stroke();

        gc.stroke_color(Color::Rgba(0.6, 0.9, 0.6, 0.8));
        gc.line_width_pixels(2.0);
        gc.stroke();
    }

    fn draw_editor(&self, gc: &mut Vec<Draw>) {
        let (width, height) = self.size.get();
        let width = width as f32;
        let height = height as f32;

        let metrics = self
            .monospace_font
            .font_metrics(self.monospace_font_size)
            .unwrap();

        let x1 = LEFT_BAR_WIDTH
            + match self.left_items.enabled {
                LeftItem::None => 0.0,
                _ => self.left_bound,
            };
        let y1 = BOTTOM_BAR_HEIGHT
            + match self.bottom_items.enabled {
                BottomItem::None => 0.0,
                _ => self.bottom_bound,
            };
        let x2 = width
            - RIGHT_BAR_WIDTH
            - match self.right_items.enabled {
                RightItem::None => 0.0,
                _ => self.right_bound,
            };
        let y2 = height - UPPER_BAR_HEIGHT;

        // gc.new_path();
        // gc.rect(x1, y1, x2, y2);
        // gc.clip();

        match self.enabled_editor {
            None => {}
            Some(idx) => {
                self.editors[idx].draw(gc, x1, y1, x2, y2 - UPPER_BAR_HEIGHT, &metrics);
            }
        }

        // gc.unclip()
    }
}

pub fn main() {
    // 'with_2d_graphics' is used to support operating systems that can't run event loops anywhere other than the main thread
    with_2d_graphics(|| {
        let serif = CanvasFontFace::from_slice(include_bytes!("Lato-Regular.ttf"));
        let monospace = CanvasFontFace::from_slice(include_bytes!("CascadiaCode.ttf"));
        let size = bind((1024, 768));
        let min_size = bind(None);
        let max_size = bind(None);
        let title = bind("Fluss".to_string());
        let is_transparent = bind(false);
        let is_visible = bind(true);
        let is_resizable = bind(true);
        let is_minimized = bind(false);
        let is_maximized = bind(false);
        let fullscreen = bind(false);
        let has_decorations = bind(true);
        let window_level = bind(WindowLevel::Normal);
        let ime_position = bind((0, 0));
        let ime_allowed = bind(false);
        let theme = bind(None);
        let cursor_position = bind((0, 0));
        let cursor_icon = bind(MousePointer::SystemDefault(CursorIcon::Default));
        let window_props = WindowProperties {
            size: BindRef::from(size.clone()),
            min_size: BindRef::from(min_size.clone()),
            max_size: BindRef::from(max_size.clone()),
            title: BindRef::from(title.clone()),
            is_transparent: BindRef::from(is_transparent.clone()),
            is_visible: BindRef::from(is_visible.clone()),
            is_resizable: BindRef::from(is_resizable.clone()),
            is_minimized: BindRef::from(is_minimized.clone()),
            is_maximized: BindRef::from(is_maximized.clone()),
            fullscreen: BindRef::from(fullscreen.clone()),
            has_decorations: BindRef::from(has_decorations.clone()),
            window_level: BindRef::from(window_level.clone()),
            ime_position: BindRef::from(ime_position.clone()),
            ime_allowed: BindRef::from(ime_allowed.clone()),
            theme: BindRef::from(theme.clone()),
            cursor_position: BindRef::from(cursor_position.clone()),
            cursor_icon: BindRef::from(cursor_icon.clone()),
        };

        let state = State {
            size,
            min_size,
            max_size,
            title,
            is_transparent,
            is_visible,
            is_resizable,
            is_minimized,
            is_maximized,
            fullscreen,
            has_decorations,
            window_level,
            ime_position,
            ime_allowed,
            theme,
            cursor_position,
            cursor_icon,
            is_exit_emmited: bind(false),
            is_needed_redraw: bind(true),

            editors: vec![EditorState {
                file: vec![
                    "fn main() {".to_string(),
                    "    println!(\"Hello, World!\");".to_string(),
                    "}".to_string(),
                ],
                name: "editor".to_string(),
                path: None,
                scroll: (0.0, 0.0),
                spans: vec![],
                cursors: vec![],
            }],
            enabled_editor: Some(0),

            input_state: InputState {
                pointer_key: bind(None),
                pointer_pos: bind((0.0, 0.0)),
                is_scroll_started: bind(false),
                prev_scroll_delta: bind((0.0, 0.0)),
                scroll_delta: bind((0.0, 0.0)),
            },

            left_bound: 100.0,
            right_bound: 100.0,
            bottom_bound: 100.0,

            left_items: LeftBar {
                enabled: LeftItem::None,
            },
            right_items: RightBar {
                enabled: RightItem::None,
            },
            bottom_items: BottomBar {
                enabled: BottomItem::None,
            },

            serif_font: serif,
            serif_font_size: 14.0,
            monospace_font: monospace,
            monospace_font_size: 14.0,
        };

        let events_state = state.clone();

        // Create a window and an event queue
        let (canvas, events) = create_drawing_window_with_events(window_props);
        // Track mouse events and render a circle centered on the current position (we use layer 1 for this so we don't have to re-render the whole canvas)
        thread::Builder::new()
            .name("Event thread".to_string())
            .spawn(move || {
                executor::block_on(async move {
                    let mut events = events;

                    // Main event loop
                    while let Some(event) = events.next().await {
                        match event {
                            DrawEvent::Resized(new_size) => {
                                // Resize the canvas
                                events_state
                                    .size
                                    .set((new_size.width as _, new_size.height as _));
                                events_state.is_needed_redraw.set(true);
                                // let (width, height) = (new_size.width, new_size.height);
                            }

                            // Track any event relating to the pointer
                            DrawEvent::CursorMoved { state } => {
                                // Draw a circle at the mouse position
                                if let Some((x, y)) = state.location_in_canvas {
                                    events_state.input_state.pointer_pos.set((x as _, y as _));
                                    events_state.is_needed_redraw.set(true);
                                }
                            }

                            DrawEvent::MouseWheel { delta, phase } => {
                                match phase {
                                    TouchPhase::Started => {}
                                    TouchPhase::Ended => {}
                                    TouchPhase::Moved => {}
                                    TouchPhase::Cancelled => {}
                                }
                                match delta {
                                    MouseScrollDelta::LineDelta(x, y) => {}
                                    MouseScrollDelta::PixelDelta(pos) => {}
                                }
                            }

                            // Ignore other events
                            _ => {}
                        }
                    }
                })
            })
            .expect("Event thread didn't started");
        while !state.is_exit_emmited.get() {
            if state.is_needed_redraw.get() {
                canvas.draw(|gc| {
                    state.draw(gc);
                });
                state.is_needed_redraw.set(false);
            }
            thread::sleep(Duration::from_secs_f32(1.0 / 60.0))
        }
    });
}
