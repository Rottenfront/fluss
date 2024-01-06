/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
mod drawer;
use drawer::Drawer;
mod elements;

use futures::{executor, prelude::*};
use rusttype::Scale;
use std::fs::File;
use std::io::Read;
use std::time::Duration;
use std::{cmp::min, path::PathBuf, sync::Arc, thread};

use flo_draw::*;
use flo_draw::{
    binding::{bind, BindRef, Binding, Bound, MutableBound},
    canvas::*,
    winit::{
        event::{MouseScrollDelta, TouchPhase},
        window::{CursorIcon, Theme, WindowLevel},
    },
};

const FRAMERATE: f32 = 60.0;
const SEC_PER_FRAME: f32 = 1.0 / FRAMERATE;

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

enum UIElement {
    Editor,
    EditorBar,
    LeftPane,
    RightPane,
    BottomPane,
    TopBar,
    LeftBar,
    RightBar,
    BottomBar,

    Notification(usize),

    HoverWindow,
    CompletionWindow,
    ActionWindow,
    DiagnosticsWindow,
    FindWindow,

    CommandPalette,
}

/*
Layer 0 - background
Layer 1 - editor & breadcrumbs & finder
Layer 2 - caret
Layer 3 - editors bar
Layer 4 - left pane
Layer 5 - right pane
Layer 6 - bottom pane
Layer 7 - top bar
Layer 8 - left bar
Layer 9 - right bar
Layer 10 - bottom bar

Layer 11 - notifications

Layer 12 - completion
Layer 13 - hover
Layer 14 - action window
Layer 15 - diagnostics
Layer 16 - find window

Layer 17 - command palette
*/

#[derive(Debug, Clone)]
struct Redraws {
    editor: Binding<bool>,
    caret: Binding<bool>,
    editors_bar: Binding<bool>,
    left_pane: Binding<bool>,
    right_pane: Binding<bool>,
    bottom_pane: Binding<bool>,
    top_bar: Binding<bool>,
    left_bar: Binding<bool>,
    right_bar: Binding<bool>,
    bottom_bar: Binding<bool>,
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
    is_needed_redraw: Redraws,
    was_resized: Binding<bool>,

    pointer_pos: Binding<(f32, f32)>,

    editors: Vec<EditorState>,
    enabled_editor: Option<usize>,

    left_bound: Binding<f32>,
    right_bound: Binding<f32>,
    bottom_bound: Binding<f32>,

    left_items: LeftBar,
    right_items: RightBar,
    bottom_items: BottomBar,

    serif_font: Arc<CanvasFontFace>,
    serif_font_size: f32,
    monospace_font: Arc<CanvasFontFace>,
    monospace_font_size: f32,
    monospace_width: f32,
}

pub const SERIF_FONT: FontId = FontId(1);
pub const MONOSPACE_FONT: FontId = FontId(2);

pub const UPPER_BAR_HEIGHT: f32 = 40.0;
pub const LEFT_BAR_WIDTH: f32 = 40.0;
pub const RIGHT_BAR_WIDTH: f32 = 40.0;
pub const BOTTOM_BAR_HEIGHT: f32 = 30.0;

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

struct Editors {
    editors: Vec<EditorState>,
    enabled: Option<usize>,
}

struct FileSystem {}

pub const DISTANCE_BETWEEN_NUMBER_AND_LINE: f32 = 20.0;

impl EditorState {
    fn draw(
        &self,
        gc: &mut Drawer,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        metrics: &FontMetrics,
        monospace_width: f32,
    ) {
        let first_line = (self.scroll.1 / metrics.height) as usize;
        let last_line = ((self.scroll.1 + (y2 - y1)) / metrics.height) as usize + 1;
        // println!("{first_line}, {last_line}");
        // gc.unclip();
        gc.new_path();
        gc.rect(x1, y1, x2, y2);
        gc.clip();
        let number = format!("{}", self.file.len());

        let x1 = DISTANCE_BETWEEN_NUMBER_AND_LINE
            + x1
            + DISTANCE_BETWEEN_NUMBER_AND_LINE
            + number.len() as f32 * monospace_width;

        // Lines render
        // delta is distance between left-top corner of first displayed line and left-top corner of editor
        let delta_y = self.scroll.1 - metrics.height * (first_line as f32);
        let y1 = y1 - delta_y + (1 - first_line) as f32 * metrics.height;
        gc.fill_color(Color::Rgba(0.0, 0.0, 0.0, 1.0));
        for i in first_line..min(last_line, self.file.len()) {
            // Render line
            gc.begin_line_layout(x1, y1 + i as f32 * metrics.height, TextAlignment::Left);
            gc.layout_text(MONOSPACE_FONT, self.file[i].clone());
            gc.draw_text_layout();
            // Render number
            gc.begin_line_layout(
                x1 - DISTANCE_BETWEEN_NUMBER_AND_LINE,
                y1 + i as f32 * metrics.height,
                TextAlignment::Right,
            );
            gc.layout_text(MONOSPACE_FONT, format!("{}", i + 1));
            gc.draw_text_layout();
        }

        gc.unclip();
    }
}

impl State {
    fn canvas_reset(&self, gc: &mut Drawer) {
        let (width, height) = self.size.get();
        // Clear the canvas and set up the coordinates
        gc.clear_canvas(Color::Rgba(1.0, 1.0, 1.0, 1.0));
        gc.canvas_height(height as _);
        gc.center_region(0.0, 0.0, width as _, height as _);
        // Load the fonts
        gc.define_font_data(SERIF_FONT, Arc::clone(&self.serif_font));
        gc.define_font_data(MONOSPACE_FONT, Arc::clone(&self.monospace_font));
        gc.set_font_size(SERIF_FONT, self.serif_font_size);
        gc.set_font_size(MONOSPACE_FONT, self.monospace_font_size);
    }
    fn draw(&self, gc: &mut Drawer) {
        if self.was_resized.get() {
            self.canvas_reset(gc);
            self.was_resized.set(false);
        }
        if self.is_needed_redraw.editor.get() {
            self.draw_editor(gc);
            self.is_needed_redraw.editor.set(true);
        }
        let (x, y) = self.pointer_pos.get();
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

    fn draw_editor(&self, gc: &mut Drawer) {
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
                _ => self.left_bound.get(),
            };
        let y1 = UPPER_BAR_HEIGHT;
        let x2 = width
            - RIGHT_BAR_WIDTH
            - match self.right_items.enabled {
                RightItem::None => 0.0,
                _ => self.right_bound.get(),
            };
        let y2 = height
            - BOTTOM_BAR_HEIGHT
            - match self.bottom_items.enabled {
                BottomItem::None => 0.0,
                _ => self.bottom_bound.get(),
            };

        // gc.new_path();
        // gc.rect(x1, y1, x2, y2);
        // gc.clip();

        match self.enabled_editor {
            None => {}
            Some(idx) => {
                self.editors[idx].draw(
                    gc,
                    x1,
                    y1 + UPPER_BAR_HEIGHT,
                    x2,
                    y2,
                    &metrics,
                    self.monospace_width,
                );
            }
        }

        // gc.unclip()
    }
}

pub fn main() {
    // 'with_2d_graphics' is used to support operating systems that can't run event loops anywhere other than the main thread
    with_2d_graphics(|| {
        let serif = include_bytes!("Lato-Regular.ttf");
        let monospace = include_bytes!("CascadiaCode.ttf");
        let monospace_font_size = 14.0;

        let monospace_width = {
            let font = rusttype::Font::try_from_bytes(monospace).unwrap();
            let scale = Scale::uniform(font.scale_for_pixel_height(monospace_font_size));

            let glyphs: Vec<_> = font.layout("0", scale, rusttype::point(0.0, 0.0)).collect();
            let min_x = glyphs
                .first()
                .map(|g| g.pixel_bounding_box().unwrap().min.x)
                .unwrap();
            let max_x = glyphs
                .last()
                .map(|g| g.pixel_bounding_box().unwrap().max.x)
                .unwrap();
            (max_x - min_x) as f32
        };

        let serif = CanvasFontFace::from_slice(serif);
        let monospace = CanvasFontFace::from_slice(monospace);
        let size = bind((1024, 768));
        let min_size = bind(None);
        let max_size = bind(None);
        let title = bind("Fluss".to_string());
        let is_transparent = bind(true);
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
            is_needed_redraw: Redraws {
                editor: bind(true),
                caret: bind(true),
                editors_bar: bind(true),
                left_pane: bind(true),
                right_pane: bind(true),
                bottom_pane: bind(true),
                left_bar: bind(true),
                right_bar: bind(true),
                top_bar: bind(true),
                bottom_bar: bind(true),
            },
            was_resized: bind(true),

            editors: vec![EditorState {
                file: {
                    let mut content = String::new();
                    File::open("src/main.rs")
                        .unwrap()
                        .read_to_string(&mut content)
                        .expect("where's the file");
                    content
                        .split("\n")
                        .collect::<Vec<&str>>()
                        .iter()
                        .map(|s| s.to_string())
                        .collect()
                },
                name: "editor".to_string(),
                path: None,
                scroll: (0.0, 0.0),
                spans: vec![],
                cursors: vec![],
            }],
            enabled_editor: Some(0),

            pointer_pos: bind((0.0, 0.0)),

            left_bound: bind(100.0),
            right_bound: bind(100.0),
            bottom_bound: bind(100.0),

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
            monospace_width,
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
                                events_state.was_resized.set(true);
                                events_state.is_needed_redraw.editor.set(true);
                                // let (width, height) = (new_size.width, new_size.height);
                            }

                            // Track any event relating to the pointer
                            DrawEvent::CursorMoved { state } => {
                                let (x, y) = state.location_in_window;
                                events_state.pointer_pos.set((x as _, y as _));
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
            canvas.draw(|gc| {
                let mut drawer = Drawer {
                    gc,
                    height: state.size.get().1 as _,
                };
                state.draw(&mut drawer);
            });
            thread::sleep(Duration::from_secs_f32(SEC_PER_FRAME))
        }
    });
}
