/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::path::PathBuf;
use futures::executor;
use futures::prelude::*;

use flo_draw::*;
use flo_draw::{binding::{bind, BindRef, Binding, Bound, MutableBound}, canvas::*};
use flo_draw::winit::window::CursorIcon;

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

struct State {
    size: Binding<(usize, usize)>,
    title: Binding<String>,
    fullscreen: Binding<bool>,
    has_decorations: Binding<bool>,
    mouse_pointer: Binding<MousePointer>,
}

pub fn main() {
    // 'with_2d_graphics' is used to support operating systems that can't run event loops anywhere other than the main thread
    let size = bind((1024, 768));
    with_2d_graphics(move || {
        let title = bind("Mouse tracking".to_string());
        let fullscreen = bind(false);
        let has_decorations = bind(true);
        let mouse_pointer = bind(MousePointer::SystemDefault(CursorIcon::Text));
        let window_props = WindowProperties {
            title: BindRef::from(title.clone()),
            size: BindRef::from(size.clone()),
            fullscreen: BindRef::from(fullscreen.clone()),
            has_decorations: BindRef::from(has_decorations.clone()),
            mouse_pointer: BindRef::from(mouse_pointer.clone()),
        };

        // Create a window and an event queue
        let (canvas, events) = create_drawing_window_with_events(window_props);

        // Render the window background on layer 0 (just a triangle)
        canvas.draw(|gc| {
            // Clear the canvas and set up the coordinates
            gc.clear_canvas(Color::Rgba(0.3, 0.2, 0.0, 1.0));
            gc.canvas_height(size.get().1 as _);
            gc.center_region(0.0, 0.0, size.get().0 as _, size.get().1 as _);

            // We'll draw some graphics to layer 0 (we can leave these alone as we track the mouse around)
            gc.layer(LayerId(0));

            // Draw a rectangle...
            gc.new_path();
            gc.move_to(0.0, 0.0);
            gc.line_to(size.get().0 as _, 0.0);
            gc.line_to(size.get().0 as _, size.get().1 as _);
            gc.line_to(0.0, size.get().1 as _);
            gc.line_to(0.0, 0.0);

            gc.fill_color(Color::Rgba(1.0, 1.0, 0.8, 1.0));
            gc.fill();

            // Draw a triangle on top
            gc.new_path();
            gc.move_to(200.0, 200.0);
            gc.line_to(800.0, 200.0);
            gc.line_to(500.0, 800.0);
            gc.line_to(200.0, 200.0);

            gc.fill_color(Color::Rgba(0.0, 0.0, 0.8, 1.0));
            gc.fill();
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
                        canvas.draw(|gc| {
                            // Clear the canvas and set up the coordinates
                            gc.clear_canvas(Color::Rgba(0.3, 0.2, 0.0, 1.0));
                            gc.canvas_height(size.get().1 as _);
                            gc.center_region(0.0, 0.0, size.get().0 as _, size.get().1 as _);

                            // We'll draw some graphics to layer 0 (we can leave these alone as we track the mouse around)
                            gc.layer(LayerId(0));

                            // Draw a rectangle...
                            gc.new_path();
                            gc.move_to(0.0, 0.0);
                            gc.line_to(size.get().0 as _, 0.0);
                            gc.line_to(size.get().0 as _, size.get().1 as _);
                            gc.line_to(0.0, size.get().1 as _);
                            gc.line_to(0.0, 0.0);

                            gc.fill_color(Color::Rgba(1.0, 1.0, 0.8, 1.0));
                            gc.fill();

                            // Draw a triangle on top
                            gc.new_path();
                            gc.move_to(200.0, 200.0);
                            gc.line_to(800.0, 200.0);
                            gc.line_to(500.0, 800.0);
                            gc.line_to(200.0, 200.0);

                            gc.fill_color(Color::Rgba(0.0, 0.0, 0.8, 1.0));
                            gc.fill();
                        });
                    }

                    // Track any event relating to the pointer
                    DrawEvent::CursorMoved { state } => {
                        // Draw a circle at the mouse position
                        if let Some((x, y)) = state.location_in_canvas {
                            canvas.draw(|gc| {
                                // Draw on layer 1 to avoid disrupting the image underneath
                                gc.layer(LayerId(1));
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
