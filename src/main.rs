mod backend;

use backend::{SkiaBackend, SkiaEnv};

use std::{
    cmp::min,
    collections::HashMap,
    path::PathBuf,
    time::{Duration, Instant},
};

use skia_safe::{
    font_style::{Slant, Weight, Width},
    svg::Dom as SvgDom,
    Canvas, ClipOp, Color, Color4f, Font, FontMgr, FontStyle, Paint, Point, Rect, Size, TextBlob,
};
use winit::{
    dpi::LogicalSize,
    event::{Event, KeyEvent, Modifiers, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey, PhysicalKey},
    window::WindowBuilder,
};

#[cfg(feature = "metal-render")]
fn main() {
    use cocoa::{
        appkit::{NSView, NSWindow},
        base::id as cocoa_id,
    };
    use core_graphics_types::geometry::CGSize;
    use foreign_types_shared::{ForeignType, ForeignTypeRef};
    use metal::{Device, MTLPixelFormat, MetalLayer};
    use objc::{
        rc::autoreleasepool,
        runtime::{NO, YES},
    };
    use skia_safe::gpu::{self, mtl, BackendRenderTarget, DirectContext, SurfaceOrigin};
    use winit::{
        platform::macos::{WindowBuilderExtMacOS, WindowExtMacOS},
        raw_window_handle::HasWindowHandle,
    };
    let app = ApplicationState {
        monospace_font: Font::new(
            FontMgr::new()
                .match_family_style(
                    "Cascadia Code PL",
                    FontStyle::new(Weight::NORMAL, Width::NORMAL, Slant::Upright),
                )
                .unwrap(),
            14.0,
        ),
    };

    let size = LogicalSize::new(800, 600);

    let events_loop = EventLoop::new().expect("Failed to create event loop");

    let window = WindowBuilder::new()
        .with_inner_size(size)
        .with_transparent(true)
        .with_titlebar_transparent(true)
        .with_title_hidden(true)
        .with_fullsize_content_view(true)
        .with_title("Skia Metal Winit Example".to_string())
        .build(&events_loop)
        .unwrap();

    let window_handle = window
        .window_handle()
        .expect("Failed to retrieve a window handle");

    let raw_window_handle = window_handle.as_raw();

    let device = Device::system_default().expect("no device found");

    let metal_layer = {
        let draw_size = window.inner_size();
        let layer = MetalLayer::new();
        layer.set_device(&device);
        layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        layer.set_presents_with_transaction(false);
        // Disabling this option allows Skia's Blend Mode to work.
        // More about: https://developer.apple.com/documentation/quartzcore/cametallayer/1478168-framebufferonly
        layer.set_framebuffer_only(false);

        unsafe {
            let view = match raw_window_handle {
                raw_window_handle::RawWindowHandle::AppKit(appkit) => appkit.ns_view.as_ptr(),
                _ => panic!("Wrong window handle type"),
            } as cocoa_id;
            // view.setTitlebarAppearsTransparent_(NO);
            view.setWantsLayer(YES);
            view.setLayer(layer.as_ref() as *const _ as _);
        }
        layer.set_drawable_size(CGSize::new(draw_size.width as f64, draw_size.height as f64));
        layer
    };

    let command_queue = device.new_command_queue();

    let backend = unsafe {
        mtl::BackendContext::new(
            device.as_ptr() as mtl::Handle,
            command_queue.as_ptr() as mtl::Handle,
            std::ptr::null(),
        )
    };

    let mut context = DirectContext::new_metal(&backend, None).unwrap();

    events_loop
        .run(move |event, window_target| {
            autoreleasepool(|| match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => window_target.exit(),
                    WindowEvent::Resized(size) => {
                        metal_layer
                            .set_drawable_size(CGSize::new(size.width as f64, size.height as f64));
                        window.request_redraw()
                    }
                    WindowEvent::RedrawRequested => {
                        if let Some(drawable) = metal_layer.next_drawable() {
                            let drawable_size = {
                                let size = metal_layer.drawable_size();
                                Size::new(size.width as scalar, size.height as scalar)
                            };

                            let mut surface = unsafe {
                                let texture_info = mtl::TextureInfo::new(
                                    drawable.texture().as_ptr() as mtl::Handle,
                                );

                                let backend_render_target = BackendRenderTarget::new_metal(
                                    (drawable_size.width as i32, drawable_size.height as i32),
                                    &texture_info,
                                );

                                gpu::surfaces::wrap_backend_render_target(
                                    &mut context,
                                    &backend_render_target,
                                    SurfaceOrigin::TopLeft,
                                    ColorType::BGRA8888,
                                    None,
                                    None,
                                )
                                .unwrap()
                            };

                            app.draw(surface.canvas());

                            context.flush_and_submit();
                            drop(surface);

                            let command_buffer = command_queue.new_command_buffer();
                            command_buffer.present_drawable(drawable);
                            command_buffer.commit();
                        }
                    }
                    _ => (),
                },
                Event::LoopExiting => {}
                _ => {}
            });
        })
        .expect("run() failed");
}

pub enum AppFocus {
    LeftDock,
    RightDock,
    BottomDock,
    /// Splitted editor part number
    /// Iif there are no splits - can be any number
    Split(usize),
}

struct ApplicationState {
    monospace_font: Font,
    test_editor: EditorState,
}

struct Bar {
    hovered_button: isize,
    enabled_button: isize,
}

#[derive(Debug, Clone)]
pub enum SpanType {
    Text,
    Comment,
    Keyword,
}

#[derive(Debug, Clone, Copy)]
struct Cursor {
    selection_pos: (usize, usize),
    position: (usize, usize),
    /// Needed to represent normal position on line when cursor switches to the line with length
    /// less than cursor's position on line
    normal_x: usize,
}

pub struct EditorState {
    file: Vec<Vec<char>>,
    name: String,
    path: Option<PathBuf>,
    scroll: (f32, f32),
    // spans: Binding<Vec<(usize, usize, usize, usize, SpanType)>>,
    cursors: Vec<Cursor>,
    tab_length: usize,
}
#[derive(PartialEq, Eq)]
pub enum Arrow {
    Left,
    Up,
    Right,
    Down,
}

#[derive(PartialEq, Eq)]
pub enum CursorInput {
    ArrLeft,
    ArrUp,
    ArrRight,
    ArrDown,
    ShiftArrLeft,
    ShiftArrUp,
    ShiftArrRight,
    ShiftArrDown,
    Backspace,
    Return,
    Delete,
    Tab,
    Character(char),
}

impl CursorInput {
    pub fn is_arrow(&self) -> bool {
        *self == CursorInput::ArrUp
            || *self == CursorInput::ArrLeft
            || *self == CursorInput::ArrDown
            || *self == CursorInput::ArrRight
    }
    pub fn is_shift_arrow(&self) -> bool {
        *self == CursorInput::ShiftArrUp
            || *self == CursorInput::ShiftArrLeft
            || *self == CursorInput::ShiftArrDown
            || *self == CursorInput::ShiftArrRight
    }
}

const DISTANCE_BETWEEN_NUMBER_AND_LINE: f32 = 20.0;
pub const UPPER_BAR_HEIGHT: f32 = 40.0;
pub const LEFT_BAR_WIDTH: f32 = 40.0;
pub const RIGHT_BAR_WIDTH: f32 = 40.0;
pub const BOTTOM_BAR_HEIGHT: f32 = 30.0;

impl EditorState {
    fn draw(&self, canvas: &Canvas, rect: Rect, monospace_font: &Font) {
        let (font_height, metrics) = monospace_font.metrics();
        let x1 = rect.left;
        let x2 = rect.right;
        let y1 = rect.top;
        let y2 = rect.bottom;
        canvas.save();
        canvas.clip_rect(&rect, Some(ClipOp::Intersect), Some(true));
        let first_line = (self.scroll.1 / font_height) as usize;
        let last_line = ((self.scroll.1 + (y2 - y1)) / font_height) as usize + 1;

        let number = format!("{}", self.file.len());
        let mut font_width = [0.0];
        monospace_font.get_widths(&[25], &mut font_width);
        let font_width = font_width[0];
        let number_len = number.len() as f32 * font_width;
        // abscisse of the start of every line in the editor
        let x1 = x1 + number_len + DISTANCE_BETWEEN_NUMBER_AND_LINE * 2.0;

        // Lines render
        // delta is distance between left-top corner of first displayed line and left-top corner of editor
        let delta_y = self.scroll.1 - font_height * (first_line as f32);
        let y1 = y1 - delta_y + (1 - first_line) as f32 * font_height;
        for i in first_line..min(last_line, self.file.len()) {
            // Render line
            if !self.file[i].is_empty() {
                canvas.draw_text_blob(
                    TextBlob::new(&String::from_iter(&self.file[i]), monospace_font).unwrap(),
                    Point::new(x1, y1 + i as f32 * font_height),
                    &Paint::new(Color4f::new(0.0, 0.0, 0.0, 1.0), None),
                );
            }
            // Render number
            let linenum = format!("{i}");
            canvas.draw_text_blob(
                TextBlob::new(&linenum, monospace_font).unwrap(),
                Point::new(
                    x1 - DISTANCE_BETWEEN_NUMBER_AND_LINE - linenum.len() as f32 * font_width,
                    y1 + i as f32 * font_height,
                ),
                &Paint::new(Color4f::new(0.0, 0.0, 0.0, 1.0), None),
            );
        }
        for cursor in &self.cursors {
            let selection_line = cursor.selection_pos.0;
            let selection_char = cursor.selection_pos.1;
            let line = cursor.position.0;
            let ch = cursor.position.1;
            let x = x1 + ch as f32 * font_width;
            let y = y1 + line as f32 * font_height + (font_height - metrics.cap_height) / 2.0;
            canvas.draw_rect(
                &Rect::from_ltrb(x - 1.0, y - font_height, x + 1.0, y),
                &Paint::new(Color4f::new(0.0, 0.0, 1.0, 1.0), None),
            );
            if selection_line == line {
                if ch != selection_char {
                    let sel_x = x1 + selection_char as f32 * font_width;
                    let (min_x, max_x) = if sel_x < x { (sel_x, x) } else { (x, sel_x) };
                    canvas.draw_rect(
                        &Rect::from_ltrb(min_x, y - font_height, max_x, y),
                        &Paint::new(Color4f::new(0.0, 0.0, 1.0, 0.3), None),
                    );
                }
            } else {
                let (min_span, max_span) = if cursor.selection_pos.0 < cursor.position.0 {
                    (cursor.selection_pos, cursor.position)
                } else {
                    (cursor.position, cursor.selection_pos)
                };
                // min span
                {
                    let x = x1 + min_span.1 as f32 * font_width;
                    let y = y1
                        + min_span.0 as f32 * font_height
                        + (font_height - metrics.cap_height) / 2.0;
                    canvas.draw_rect(
                        &Rect::from_ltrb(x, y - font_height, x2, y),
                        &Paint::new(Color4f::new(0.0, 0.0, 1.0, 0.3), None),
                    );
                }
                // max_span
                {
                    let x = x1 + max_span.1 as f32 * font_width;
                    let y = y1
                        + max_span.0 as f32 * font_height
                        + (font_height - metrics.cap_height) / 2.0;
                    canvas.draw_rect(
                        &Rect::from_ltrb(x1, y - font_height, x, y),
                        &Paint::new(Color4f::new(0.0, 0.0, 1.0, 0.3), None),
                    );
                }
                // filled lines
                if max_span.0 - min_span.0 > 1 {
                    let min_y = y1
                        + min_span.0 as f32 * font_height
                        + (font_height - metrics.cap_height) / 2.0;
                    let max_y = y1
                        + (max_span.0 - 1) as f32 * font_height
                        + (font_height - metrics.cap_height) / 2.0;
                    canvas.draw_rect(
                        &Rect::from_ltrb(x1, min_y, x2, max_y),
                        &Paint::new(Color4f::new(0.0, 0.0, 1.0, 0.3), None),
                    );
                }
            }
        }
        canvas.restore();
    }

    fn draw_cursor(&self, cursor: &Cursor, x1: f32, x2: f32, y_first_line: f32) {}

    fn cursor_up(&mut self, cursor_id: usize) {
        let cursor = &mut self.cursors[cursor_id];
        if cursor.position.0 != 0 {
            cursor.position.0 -= 1;
            cursor.position.1 = min(self.file[cursor.position.0].len(), cursor.normal_x);
            cursor.selection_pos = cursor.position;
        }
    }

    fn cursor_down(&mut self, cursor_id: usize) {
        let cursor = &mut self.cursors[cursor_id];
        if cursor.position.0 < self.file.len() - 1 {
            cursor.position.0 += 1;
            cursor.position.1 = min(self.file[cursor.position.0].len(), cursor.normal_x);
            cursor.selection_pos = cursor.position;
        }
    }

    fn cursor_left(&mut self, cursor_id: usize) {
        let cursor = &mut self.cursors[cursor_id];
        if cursor.selection_pos != cursor.position {
            // adjust selection to the left side
            if cursor.selection_pos.0 < cursor.position.0
                || (cursor.selection_pos.0 == cursor.position.0
                    && cursor.selection_pos.1 < cursor.position.1)
            {
                cursor.position = cursor.selection_pos;
            } else {
                cursor.selection_pos = cursor.position;
            }
        } else {
            // decrease cursor index
            if cursor.position.1 == 0 {
                if cursor.position.0 != 0 {
                    cursor.position.0 -= 1;
                    cursor.position.1 = self.file[cursor.position.0].len();
                }
            } else {
                cursor.position.1 -= 1;
            }
        }
        cursor.normal_x = cursor.position.1;
        cursor.selection_pos = cursor.position;
    }

    fn cursor_right(&mut self, cursor_id: usize) {
        let cursor = &mut self.cursors[cursor_id];
        if cursor.selection_pos != cursor.position {
            // adjust selection to the right side
            if cursor.selection_pos.0 < cursor.position.0
                || (cursor.selection_pos.0 == cursor.position.0
                    && cursor.selection_pos.1 < cursor.position.1)
            {
                cursor.selection_pos = cursor.position;
            } else {
                cursor.position = cursor.selection_pos;
            }
        } else {
            // increase cursor index
            if cursor.position.0 < self.file.len() {
                if cursor.position.1 == self.file[cursor.position.0].len() {
                    if cursor.position.0 < self.file.len() - 1 {
                        cursor.position.0 += 1;
                        cursor.position.1 = 0;
                    }
                } else {
                    cursor.position.1 += 1;
                }
            }
        }
        cursor.normal_x = cursor.position.1;
        cursor.selection_pos = cursor.position;
    }

    fn cursor_selection_up(&mut self, cursor_id: usize) {
        let cursor = &mut self.cursors[cursor_id];
        if cursor.position.0 != 0 {
            cursor.position.0 -= 1;
            cursor.position.1 = min(self.file[cursor.position.0].len(), cursor.normal_x);
        }
    }
    fn cursor_selection_down(&mut self, cursor_id: usize) {
        let cursor = &mut self.cursors[cursor_id];
        if cursor.position.0 < self.file.len() - 1 {
            cursor.position.0 += 1;
            cursor.position.1 = min(self.file[cursor.position.0].len(), cursor.normal_x);
        }
    }
    fn cursor_selection_left(&mut self, cursor_id: usize) {
        let cursor = &mut self.cursors[cursor_id];
        // decrease cursor index
        if cursor.position.1 == 0 {
            if cursor.position.0 != 0 {
                cursor.position.0 -= 1;
                cursor.position.1 = self.file[cursor.position.0].len();
            }
        } else {
            cursor.position.1 -= 1;
        }
        cursor.normal_x = cursor.position.1;
    }
    fn cursor_selection_right(&mut self, cursor_id: usize) {
        let cursor = &mut self.cursors[cursor_id];
        // increase/decrease cursor index
        if cursor.position.0 < self.file.len() {
            if cursor.position.1 == self.file[cursor.position.0].len() {
                if cursor.position.0 < self.file.len() - 1 {
                    cursor.position.0 += 1;
                    cursor.position.1 = 0;
                }
            } else {
                cursor.position.1 += 1;
            }
        }
        cursor.normal_x = cursor.position.1;
    }

    fn cursor_handle_return(&mut self, cursor_id: usize) {
        let mut cursor = self.cursors[cursor_id];
        if cursor.selection_pos.0 != cursor.position.0 {
            let (min_span, max_span) = if cursor.selection_pos.0 < cursor.position.0 {
                (cursor.selection_pos, cursor.position)
            } else {
                (cursor.position, cursor.selection_pos)
            };
            self.cursor_remove_absorbed_lines(&cursor);
            self.file[min_span.0] = (&self.file[min_span.0][..min_span.1]).to_owned();
            // because of removed lines cursor pointer with max spin now has line pointer on min_spin.line + 1
            self.file[min_span.0 + 1] = (&self.file[min_span.0 + 1][max_span.1..]).to_owned();
            cursor.position = (min_span.0 + 1, 0);
        } else {
            let (min_span, max_span) = if cursor.selection_pos.1 < cursor.position.1 {
                (cursor.selection_pos.1, cursor.position.1)
            } else {
                (cursor.position.1, cursor.selection_pos.1)
            };
            let first_line = (&self.file[cursor.position.0][..min_span]).to_owned();
            let second_line = (&self.file[cursor.position.0][max_span..]).to_owned();
            self.file[cursor.position.0] = first_line.to_owned();
            self.file.insert(cursor.position.0 + 1, second_line);
            cursor.position.0 += 1;
            cursor.position.1 = 0;
        }

        self.cursor_sync_cords(cursor, cursor_id);
    }

    fn cursor_handle_tab(&mut self, cursor_id: usize) {
        let cursor = &mut self.cursors[cursor_id];
        if cursor.selection_pos.0 != cursor.position.0 {
            let (min_span, max_span) = if cursor.selection_pos.0 < cursor.position.0 {
                (cursor.selection_pos, cursor.position)
            } else {
                (cursor.position, cursor.selection_pos)
            };
            for i in min_span.0..=max_span.0 {
                for _ in 0..self.tab_length {
                    self.file[i].insert(0, ' ');
                }
            }
            cursor.position.1 += self.tab_length;
            cursor.selection_pos.1 += self.tab_length;
        } else {
            let (min_span, max_span) = if cursor.selection_pos.1 < cursor.position.1 {
                (cursor.selection_pos.1, cursor.position.1)
            } else {
                (cursor.position.1, cursor.selection_pos.1)
            };
            if max_span - min_span > 0 {
                for _ in 0..self.tab_length {
                    self.file[cursor.position.0].insert(cursor.position.1, ' ');
                }
                cursor.position.1 += self.tab_length;
                cursor.selection_pos.1 += self.tab_length;
            } else {
                let len = self.tab_length - cursor.position.1 % self.tab_length;
                for _ in 0..len {
                    self.file[cursor.position.0].insert(cursor.position.1, ' ');
                }
                cursor.position.1 += len;
                cursor.selection_pos.1 = cursor.position.1;
            }
        }
        cursor.normal_x = cursor.position.1;
    }

    fn cursor_handle_delete(&mut self, cursor_id: usize) {
        let mut cursor = self.cursors[cursor_id];

        if cursor.selection_pos.0 != cursor.position.0 {
            let (min_span, max_span) = if cursor.selection_pos.0 < cursor.position.0 {
                (cursor.selection_pos, cursor.position)
            } else {
                (cursor.position, cursor.selection_pos)
            };
            self.cursor_remove_absorbed_lines(&cursor);
            while self.file[min_span.0].len() > min_span.1 {
                self.file[min_span.0].pop();
            }
            let mut next_line = vec![];
            for i in max_span.1..self.file[min_span.0 + 1].len() {
                next_line.push(self.file[min_span.0 + 1][i]);
            }
            self.file[min_span.0].append(&mut next_line);
            self.file.remove(min_span.0 + 1);
            cursor.position = min_span;
        } else {
            let (min_span, max_span) = if cursor.selection_pos.1 < cursor.position.1 {
                (cursor.selection_pos.1, cursor.position.1)
            } else {
                (cursor.position.1, cursor.selection_pos.1)
            };
            if min_span == max_span {
                if min_span == self.file[cursor.position.0].len() {
                    if cursor.position.0 < self.file.len() - 1 {
                        let mut line = self.file.remove(cursor.position.0 + 1);
                        self.file[cursor.position.0].append(&mut line);
                    }
                } else {
                    self.file[cursor.position.0].remove(min_span);
                }
            } else {
                for _ in min_span..max_span {
                    self.file[cursor.position.0].remove(min_span);
                }
                cursor.position.1 = min_span;
            }
        }

        self.cursor_sync_cords(cursor, cursor_id);
    }

    fn cursor_handle_backspace(&mut self, cursor_id: usize) {
        let mut cursor = self.cursors[cursor_id];

        if cursor.selection_pos.0 != cursor.position.0 {
            let (min_span, max_span) = if cursor.selection_pos.0 < cursor.position.0 {
                (cursor.selection_pos, cursor.position)
            } else {
                (cursor.position, cursor.selection_pos)
            };
            self.cursor_remove_absorbed_lines(&cursor);
            while self.file[min_span.0].len() > min_span.1 {
                self.file[min_span.0].pop();
            }
            let mut next_line = vec![];
            for i in max_span.1..self.file[min_span.0 + 1].len() {
                next_line.push(self.file[min_span.0 + 1][i]);
            }
            self.file[min_span.0].append(&mut next_line);
            self.file.remove(min_span.0 + 1);
            cursor.position = min_span;
        } else {
            let (min_span, max_span) = if cursor.selection_pos.1 < cursor.position.1 {
                (cursor.selection_pos.1, cursor.position.1)
            } else {
                (cursor.position.1, cursor.selection_pos.1)
            };
            if min_span == max_span {
                if min_span == 0 {
                    if cursor.position.0 > 0 {
                        let mut line = self.file.remove(cursor.position.0);
                        cursor.position = (
                            cursor.position.0 - 1,
                            self.file[cursor.position.0 - 1].len(),
                        );
                        self.file[cursor.position.0].append(&mut line);
                        cursor.selection_pos = cursor.position;
                    }
                } else {
                    self.file[cursor.position.0].remove(min_span - 1);
                    cursor.position.1 -= 1;
                }
            } else {
                for _ in min_span..max_span {
                    self.file[cursor.position.0].remove(min_span);
                }
                cursor.position.1 = min_span;
            }
        }

        self.cursor_sync_cords(cursor, cursor_id)
    }

    fn cursor_handle_char(&mut self, cursor_id: usize, ch: char) {
        let mut cursor = self.cursors[cursor_id];

        if cursor.selection_pos.0 != cursor.position.0 {
            let (min_span, max_span) = if cursor.selection_pos.0 < cursor.position.0 {
                (cursor.selection_pos, cursor.position)
            } else {
                (cursor.position, cursor.selection_pos)
            };
            self.cursor_remove_absorbed_lines(&cursor);
            while self.file[min_span.0].len() > min_span.1 {
                self.file[min_span.0].pop();
            }
            let mut next_line = vec![ch];
            for i in max_span.1..self.file[min_span.0 + 1].len() {
                next_line.push(self.file[min_span.0 + 1][i]);
            }
            self.file[min_span.0].append(&mut next_line);
            self.file.remove(min_span.0 + 1);
            cursor.position = (min_span.0, min_span.1 + 1);
        } else {
            let (min_span, max_span) = if cursor.selection_pos.1 < cursor.position.1 {
                (cursor.selection_pos.1, cursor.position.1)
            } else {
                (cursor.position.1, cursor.selection_pos.1)
            };
            for _ in min_span..max_span {
                self.file[cursor.position.0].remove(min_span);
            }
            self.file[cursor.position.0].insert(min_span, ch);
            // .replace_range(min_span..max_span, &String::from(ch));
            cursor.position.1 = min_span + 1;
        }

        self.cursor_sync_cords(cursor, cursor_id)
    }

    fn cursor_remove_absorbed_lines(&mut self, cursor: &Cursor) {
        let (min_span, max_span) = if cursor.selection_pos.0 < cursor.position.0 {
            (cursor.selection_pos.0, cursor.position.0)
        } else {
            (cursor.position.0, cursor.selection_pos.0)
        };
        for _ in (min_span + 1)..max_span {
            self.file.remove(min_span + 1);
        }
    }

    fn cursor_sync_cords(&mut self, mut cursor: Cursor, cursor_id: usize) {
        cursor.selection_pos = cursor.position;
        cursor.normal_x = cursor.position.1;

        self.cursors[cursor_id] = cursor;
    }

    pub fn handle_cursor_input(&mut self, cursor_id: usize, input: Key, modifiers: &Modifiers) {
        match input {
            Key::Named(key) => match key {
                NamedKey::ArrowUp => {
                    if modifiers.state().shift_key() {
                        self.cursor_selection_up(cursor_id)
                    } else {
                        self.cursor_up(cursor_id)
                    }
                }
                NamedKey::ArrowDown => {
                    if modifiers.state().shift_key() {
                        self.cursor_selection_down(cursor_id)
                    } else {
                        self.cursor_down(cursor_id)
                    }
                }
                NamedKey::ArrowLeft => {
                    if modifiers.state().shift_key() {
                        self.cursor_selection_left(cursor_id)
                    } else {
                        self.cursor_left(cursor_id)
                    }
                }
                NamedKey::ArrowRight => {
                    if modifiers.state().shift_key() {
                        self.cursor_selection_right(cursor_id)
                    } else {
                        self.cursor_right(cursor_id)
                    }
                }
                NamedKey::Tab => self.cursor_handle_tab(cursor_id),
                NamedKey::Delete => self.cursor_handle_delete(cursor_id),
                NamedKey::Enter => self.cursor_handle_return(cursor_id),
                NamedKey::Backspace => self.cursor_handle_backspace(cursor_id),
                _ => {}
            },
            Key::Character(ref ch) => {
                let v = ch.chars().collect::<Vec<char>>();
                self.cursor_handle_char(cursor_id, v[0])
            }
            _ => {}
        }
    }
}

impl ApplicationState {
    /// Renders a rectangle that occupies exactly half of the canvas
    fn draw(&self, canvas: &Canvas) {
        let canvas_size = Size::from(canvas.base_layer_size());

        canvas.clear(Color::WHITE);

        self.test_editor.draw(
            canvas,
            Rect::from_ltrb(
                LEFT_BAR_WIDTH,
                UPPER_BAR_HEIGHT,
                canvas_size.width - RIGHT_BAR_WIDTH,
                canvas_size.height - BOTTOM_BAR_HEIGHT,
            ),
            &self.monospace_font,
        );
    }
}

fn main() {
    let mut app = ApplicationState {
        monospace_font: Font::new(
            FontMgr::new()
                .match_family_style(
                    "Cascadia Code PL",
                    FontStyle::new(Weight::BOLD, Width::NORMAL, Slant::Upright),
                )
                .unwrap(),
            13.0,
        ),
        test_editor: EditorState {
            file: vec![
                "fn main() {".chars().collect(),
                "    println!(\"Hello, World!\");".chars().collect(),
                "}".chars().collect(),
            ],
            name: "name".to_string(),
            path: None,
            scroll: (0.0, 0.0),
            cursors: vec![Cursor {
                position: (0, 0),
                selection_pos: (0, 0),
                normal_x: 0,
            }],
            tab_length: 4,
        },
    };

    // let font_mgr = FontMgr::new();
    // let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" height = "100" width = "100">
    //     <path d="M30,1h40l29,29v40l-29,29h-40l-29-29v-40z" stroke="#;000" fill="none"/>
    //     <path d="M31,3h38l28,28v38l-28,28h-38l-28-28v-38z" fill="#a23"/>
    //     <text x="50" y="68" font-size="48" fill="#FFF" text-anchor="middle"><![CDATA[410]]></text>
    //     </svg>"##;
    // let dom = SvgDom::from_str(svg, font_mgr).unwrap();

    let el = EventLoop::new().expect("Failed to create event loop");
    let winit_window_builder = WindowBuilder::new()
        .with_title("rust-skia-gl-window")
        .with_inner_size(LogicalSize::new(800, 800))
        .with_transparent(true)
        .with_blur(true);

    let mut env = SkiaEnv::new(winit_window_builder, &el);
    let mut previous_frame_start = Instant::now();
    let mut modifiers = Modifiers::default();
    let expected_frame_length_seconds = 1.0 / 60.0;
    let frame_duration = Duration::from_secs_f32(expected_frame_length_seconds);
    let mut frame = 0usize;

    let mut keypress_map: HashMap<PhysicalKey, bool> = HashMap::new();
    el.run(move |event, window_target| {
        let frame_start = Instant::now();
        let mut draw_frame = false;

        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::CloseRequested => {
                    // app.close();
                    window_target.exit();
                    return;
                }
                WindowEvent::Resized(physical_size) => {
                    env.on_resize(physical_size);
                }
                WindowEvent::ModifiersChanged(new_modifiers) => modifiers = new_modifiers,
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            logical_key,
                            physical_key,
                            repeat,
                            ..
                        },
                    ..
                } => {
                    if modifiers.state().super_key() && logical_key == "q" {
                        window_target.exit();
                    }
                    if !keypress_map.contains_key(&physical_key) {
                        keypress_map.insert(physical_key, false);
                    }

                    if !keypress_map[&physical_key] || repeat {
                        if !modifiers.state().super_key() && !modifiers.state().control_key() {
                            for id in 0..app.test_editor.cursors.len() {
                                app.test_editor.handle_cursor_input(
                                    id,
                                    logical_key.clone(),
                                    &modifiers,
                                );
                            }
                        }
                    }
                    if !repeat {
                        let b = keypress_map.remove(&physical_key).unwrap();
                        keypress_map.insert(physical_key, !b);
                    }
                    frame = frame.saturating_sub(10);
                    env.request_redraw();
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
            frame += 1;
            let canvas = env.canvas();
            app.draw(&canvas);
            // renderer::render_frame(frame % 360, 12, 60, canvas);
            env.draw();
        }

        window_target.set_control_flow(ControlFlow::WaitUntil(
            previous_frame_start + frame_duration,
        ))
    })
    .expect("run() failed");
}

#[cfg(all(feature = "d3d-render", not(feature = "gl-render")))]
fn main() -> anyhow::Result<()> {
    // NOTE: Most of code is from https://github.com/microsoft/windows-rs/blob/02db74cf5c4796d970e6d972cdc7bc3967380079/crates/samples/windows/direct3d12/src/main.rs

    use std::ptr;

    use anyhow::Result;
    use skia_safe::{
        gpu::{
            d3d::{BackendContext, TextureResourceInfo},
            surfaces, BackendRenderTarget, DirectContext, Protected, SurfaceOrigin,
        },
        paint, Color, ColorType, Paint, Rect,
    };
    use windows::{
        core::ComInterface,
        Win32::{
            Foundation::HWND,
            Graphics::{
                Direct3D::D3D_FEATURE_LEVEL_11_0,
                Direct3D12::{
                    D3D12CreateDevice, ID3D12CommandQueue, ID3D12DescriptorHeap, ID3D12Device,
                    ID3D12Resource, D3D12_COMMAND_LIST_TYPE_DIRECT, D3D12_COMMAND_QUEUE_DESC,
                    D3D12_COMMAND_QUEUE_FLAG_NONE, D3D12_CPU_DESCRIPTOR_HANDLE,
                    D3D12_DESCRIPTOR_HEAP_DESC, D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                    D3D12_RESOURCE_STATE_COMMON,
                },
                Dxgi::{
                    Common::{
                        DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC,
                        DXGI_STANDARD_MULTISAMPLE_QUALITY_PATTERN,
                    },
                    CreateDXGIFactory1, IDXGIAdapter1, IDXGIFactory4, IDXGISwapChain3,
                    DXGI_ADAPTER_FLAG, DXGI_ADAPTER_FLAG_NONE, DXGI_ADAPTER_FLAG_SOFTWARE,
                    DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_EFFECT_FLIP_DISCARD,
                    DXGI_USAGE_RENDER_TARGET_OUTPUT,
                },
            },
        },
    };
    use winit::{
        event::{Event, WindowEvent},
        keyboard::{Key, NamedKey},
    };

    let event_loop = winit::event_loop::EventLoop::new()?;
    let winit_window_builder = winit::window::WindowBuilder::new()
        .with_title("rust-skia-gl-window")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 800));

    let window = winit_window_builder.build(&event_loop)?;

    const FRAME_COUNT: u32 = 2;
    let id: u64 = window.id().into();
    let hwnd = HWND(id as isize);

    let factory = unsafe { CreateDXGIFactory1::<IDXGIFactory4>() }?;
    let adapter = get_hardware_adapter(&factory)?;

    let mut device: Option<ID3D12Device> = None;
    unsafe { D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_11_0, &mut device) }?;
    let device = device.unwrap();

    let command_queue = unsafe {
        device.CreateCommandQueue::<ID3D12CommandQueue>(&D3D12_COMMAND_QUEUE_DESC {
            Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
            Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
            ..Default::default()
        })
    }?;

    let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
        BufferCount: FRAME_COUNT,
        Width: window.inner_size().width,
        Height: window.inner_size().height,
        Format: DXGI_FORMAT_R8G8B8A8_UNORM,
        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
        SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            ..Default::default()
        },
        ..Default::default()
    };

    let swap_chain: IDXGISwapChain3 = unsafe {
        factory.CreateSwapChainForHwnd(&command_queue, hwnd, &swap_chain_desc, None, None)?
    }
    .cast()?;

    let frame_index = unsafe { swap_chain.GetCurrentBackBufferIndex() };

    let rtv_heap: ID3D12DescriptorHeap = unsafe {
        device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
            NumDescriptors: FRAME_COUNT,
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            ..Default::default()
        })
    }?;

    let rtv_descriptor_size =
        unsafe { device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV) } as usize;

    let rtv_handle = D3D12_CPU_DESCRIPTOR_HANDLE {
        ptr: unsafe { rtv_heap.GetCPUDescriptorHandleForHeapStart() }.ptr
            + frame_index as usize * rtv_descriptor_size,
    };

    let render_targets: Vec<ID3D12Resource> = {
        let mut render_targets = vec![];
        for i in 0..FRAME_COUNT {
            let render_target: ID3D12Resource = unsafe { swap_chain.GetBuffer(i)? };
            unsafe {
                device.CreateRenderTargetView(
                    &render_target,
                    None,
                    D3D12_CPU_DESCRIPTOR_HANDLE {
                        ptr: rtv_handle.ptr + i as usize * rtv_descriptor_size,
                    },
                )
            };
            render_targets.push(render_target);
        }
        render_targets
    };

    let backend_context = BackendContext {
        adapter,
        device: device.clone(),
        queue: command_queue,
        memory_allocator: None,
        protected_context: Protected::No,
    };

    let mut context = unsafe { DirectContext::new_d3d(&backend_context, None).unwrap() };

    let mut surfaces = render_targets
        .iter()
        .map(|render_target| {
            let backend_render_target = BackendRenderTarget::new_d3d(
                (
                    window.inner_size().width as i32,
                    window.inner_size().height as i32,
                ),
                &TextureResourceInfo {
                    resource: render_target.clone(),
                    alloc: None,
                    resource_state: D3D12_RESOURCE_STATE_COMMON,
                    format: DXGI_FORMAT_R8G8B8A8_UNORM,
                    sample_count: 1,
                    level_count: 0,
                    sample_quality_pattern: DXGI_STANDARD_MULTISAMPLE_QUALITY_PATTERN,
                    protected: Protected::No,
                },
            );

            surfaces::wrap_backend_render_target(
                &mut context,
                &backend_render_target,
                SurfaceOrigin::BottomLeft,
                ColorType::RGBA8888,
                None,
                None,
            )
            .ok_or(anyhow::anyhow!("wrap_backend_render_target failed"))
        })
        .collect::<Result<Vec<_>>>()?;

    fn get_hardware_adapter(factory: &IDXGIFactory4) -> Result<IDXGIAdapter1> {
        for i in 0.. {
            let adapter = unsafe { factory.EnumAdapters1(i)? };

            let mut desc = Default::default();
            unsafe { adapter.GetDesc1(&mut desc)? };

            if (DXGI_ADAPTER_FLAG(desc.Flags as i32) & DXGI_ADAPTER_FLAG_SOFTWARE)
                != DXGI_ADAPTER_FLAG_NONE
            {
                // Don't select the Basic Render Driver adapter.
                continue;
            }

            // Check to see whether the adapter supports Direct3D 12, but don't create the actual
            // device yet.
            if unsafe {
                D3D12CreateDevice(
                    &adapter,
                    D3D_FEATURE_LEVEL_11_0,
                    ptr::null_mut::<Option<ID3D12Device>>(),
                )
            }
            .is_ok()
            {
                return Ok(adapter);
            }
        }

        unreachable!()
    }

    let mut skia_context = context;

    println!("Skia initialized with {} surfaces.", surfaces.len());
    println!("Use Arrow Keys to move the rectangle.");

    let mut next_surface_index = 0;

    struct State {
        x: f32,
        y: f32,
    }

    let mut render = |state: &State| {
        let this_index = next_surface_index;
        next_surface_index = (next_surface_index + 1) % surfaces.len();

        let surface = &mut surfaces[this_index];
        let canvas = surface.canvas();

        // canvas.clear(Color::BLUE);

        // let mut paint = Paint::default();
        // paint.set_color(Color::RED);
        // paint.set_style(paint::Style::StrokeAndFill);
        // paint.set_anti_alias(true);
        // paint.set_stroke_width(10.0);

        // canvas.draw_rect(Rect::from_xywh(state.x, state.y, 200.0, 200.0), &paint);
        draw(&canvas);
        skia_context.flush_surface(surface);

        skia_context.submit(None);

        unsafe { swap_chain.Present(1, 0).ok().unwrap() };

        // NOTE: If you get some error when you render, you can check it with:
        // unsafe {
        //     device.GetDeviceRemovedReason().ok().unwrap();
        // }
    };

    enum ControlFlow {
        Continue,
        Exit,
    }

    use ControlFlow::*;

    let mut handle_event = |event, state: &mut State| match event {
        WindowEvent::RedrawRequested => {
            render(state);
            Continue
        }
        WindowEvent::KeyboardInput { event, .. } => {
            match event.logical_key {
                Key::Named(NamedKey::ArrowLeft) => state.x -= 10.0,
                Key::Named(NamedKey::ArrowRight) => state.x += 10.0,
                Key::Named(NamedKey::ArrowUp) => state.y += 10.0,
                Key::Named(NamedKey::ArrowDown) => state.y -= 10.0,
                Key::Named(NamedKey::Escape) => return Exit,
                _ => {}
            }

            render(state);
            Continue
        }
        WindowEvent::CloseRequested => Exit,
        _ => Continue,
    };

    let mut state = State { x: 100.0, y: 100.0 };

    event_loop.run(move |event, window| {
        if let Event::WindowEvent { event, .. } = event {
            match handle_event(event, &mut state) {
                Continue => {}
                Exit => window.exit(),
            }
        }
    })?;

    Ok(())
}
