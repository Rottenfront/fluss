/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::cmp::min;
use std::path::PathBuf;
use std::sync::Arc;
use futures::executor;
use futures::prelude::*;

use flo_draw::*;
use flo_draw::{binding::{bind, BindRef, Binding, Bound, MutableBound}, canvas::*};
use flo_draw::winit::window::{CursorIcon, Theme, WindowLevel};

enum SpanType {
    Text,
    Comment,
    Keyword,
}

struct EditorState {
    file: Vec<String>,
    name: String,
    path: Option<PathBuf>,
    scroll: (f64, f64),
    spans: Vec<(usize, usize, usize, usize, SpanType)>,
    cursors: Vec<(usize, usize, usize, usize)>,
}

struct FSItem {
    name: String,
    is_folded: bool,
    level: usize,
}

struct FSState {
    root: PathBuf,
    items: Vec<FSItem>,
}

enum LeftItem {
    None,
    FS,
}

struct LeftBar {
    enabled: LeftItem,
    fs: FSState,
}

enum RightItem {
    None,
}

struct RightBar {
    enabled: RightItem,
}

enum BottomItem {
    None,
    Terminal,
}

struct TermState {
}

struct BottomBar {
    enabled: BottomItem,
    terminal: TermState,
}

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

    editors: Vec<EditorState>,
    enabled_editor: Option<usize>,

    left_bound: f64,
    right_bound: f64,
    bottom_bound: f64,

    left_items: LeftBar,
    right_items: RightBar,
    bottom_items: BottomBar,

    canvas: DrawingTarget,

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
        let first_line = (self.scroll.1 as f32 / metrics.height) as usize;
        let last_line = ((self.scroll.1 + (y1 - y2)) as f32 / metrics.height) as usize + 1;

        gc.new_path();
        gc.rect(x1, y1, x2, y2);
        gc.clip();


        let delta_y = self.scroll.1 - metrics.height * first_line;
        let y2 = y2 + delta_y;
        for i in first_line..min(last_line, self.file.len()) {
            gc.fill_text(self.file[i].as_str(), x1, y2 + (i - first_line) as f32 * metrics.height, metrics.em_size, Color::Rgba(0.0, 0.0, 0.0, 1.0));
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
    }

    fn draw_editor(&self, gc: &mut Vec<Draw>) {
        let (width, height) = self.size.get();
        let width = width as _;
        let height = height as _;

        let metrics = self.monospace_font.font_metrics(self.monospace_font_size).unwrap();

        let x1 = LEFT_BAR_WIDTH + match self.left_items.enabled {
            LeftItem::None => 0.0,
            _ => self.left_bound,
        };
        let y1 = BOTTOM_BAR_HEIGHT + match self.bottom_items.enabled {
            BottomItem::None => 0.0,
            _ => self.bottom_bound,
        };
        let x2 = width - RIGHT_BAR_WIDTH - match self.right_items.enabled {
            RightItem::None => 0.0,
            _ => self.right_bound,
        };
        let y2 = height - UPPER_BAR_HEIGHT;

        gc.new_path();
        gc.rect(x1, y1, x2, y2);
        gc.clip();

        match self.enabled_editor {
            None => {},
            Some(idx) => {
                self.editors[idx].draw(gc, x1, y1, x2, y2 - UPPER_BAR_HEIGHT, &metrics);
            },
        }

        gc.unclip()
    }
}

fn draw(gc: &mut Vec<Draw>, width: f32, height: f32, serif: &Arc<CanvasFontFace>, monospace: &Arc<CanvasFontFace>) {
    // Load the fonts
    gc.define_font_data(FontId(1), Arc::clone(&serif));
    gc.define_font_data(FontId(2), Arc::clone(&monospace));
    gc.set_font_size(FontId(1), 14.0);
    gc.set_font_size(FontId(2), 14.0);


    gc.new_path();
    gc.rect(0.0, 0.0, width, 50 as _);
    gc.rect(0.0, height - 50.0, width, height);

    gc.fill_color(Color::Rgba(1.0, 1.0, 0.8, 1.0));
    gc.fill();

    // Draw some text with layout in these fonts
    gc.fill_color(Color::Rgba(0.0, 0.0, 0.6, 1.0));

    gc.begin_line_layout(14.0, 700.0 - 100.0, TextAlignment::Left);
    gc.layout_text(FontId(1), "It's also possible to alter ".to_string());
    gc.set_font_size(FontId(1), 36.0);
    gc.layout_text(FontId(1), "sizes".to_string());
    gc.set_font_size(FontId(1), 14.0);
    gc.layout_text(FontId(1), " during layout ".to_string());
    gc.draw_text_layout();
}

pub fn main() {
    // 'with_2d_graphics' is used to support operating systems that can't run event loops anywhere other than the main thread
    with_2d_graphics(|| {
        let serif = CanvasFontFace::from_slice(include_bytes!("Lato-Regular.ttf"));
        let monospace = CanvasFontFace::from_slice(include_bytes!("CascadiaCode.ttf"));
        let size = bind((1024, 768));
        let min_size = bind(None);
        let max_size = bind(None);
        let title = bind("Mouse tracking".to_string());
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

        // Create a window and an event queue
        let (canvas, events) = create_drawing_window_with_events(window_props);
        let (width, height) = size.get();
        let width = width as f32;
        let height = height as f32;
        canvas.draw(|gc| {
            // Set up the canvas
            gc.canvas_height(height);
            gc.center_region(0.0, 0.0, width, height);
            gc.layer(LayerId(0));
            draw(gc, width, height, &serif, &monospace);
        });
        // Track mouse events and render a circle centered on the current position (we use layer 1 for this so we don't have to re-render the whole canvas)
        executor::block_on(async move {
            let mut events = events;

            // Main event loop
            while let Some(event) = events.next().await {
                match event {
                    DrawEvent::Resized(new_size) => {
                        // Resize the canvas
                        size.set((new_size.width as _, new_size.height as _));
                        let (width, height) = (new_size.width, new_size.height);
                        canvas.draw(|gc| {
                            // Clear the canvas and set up the coordinates
                            gc.clear_canvas(Color::Rgba(1.0, 1.0, 1.0, 1.0));
                            gc.canvas_height(height as _);
                            gc.center_region(0.0, 0.0, width as _, height as _);

                            gc.layer(LayerId(0));
                            gc.clear_layer();
                            draw(gc, width as _, height as _, &serif, &monospace);
                        });
                    }

                    // Track any event relating to the pointer
                    DrawEvent::CursorMoved { state } => {
                        // Draw a circle at the mouse position
                        if let Some((x, y)) = state.location_in_canvas {
                            canvas.draw(|gc| {
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
                            });
                        }
                    }

                    // Ignore other events
                    _ => {}
                }
            }
        })
    });
}
