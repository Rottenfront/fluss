use binding::{Binding, Bound, MutableBound};

use std::{cmp::min, path::PathBuf, time::Instant};

use skia_safe::{Canvas, ClipOp, Color4f, Font, Paint, Point, RRect, Rect, TextBlob};
use winit::{
    event::{Ime, Modifiers},
    keyboard::{Key, NamedKey},
};

use crate::{canvas::Drawer, MONOSPACE_FONT_ID};
#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub selection_pos: (usize, usize),
    pub position: (usize, usize),
    /// Needed to represent normal position on line when cursor switches to the line with length
    /// less than cursor's position on line
    pub normal_x: usize,
    pub last_ime_len: usize,
}

pub struct EditorState {
    pub file: Vec<Vec<char>>,
    pub name: String,
    pub path: Option<PathBuf>,
    pub scroll: (f32, f32),
    // spans: Binding<Vec<(usize, usize, usize, usize, SpanType)>>,
    pub cursors: Vec<Cursor>,
    pub tab_length: usize,
    pub line_height: Binding<f32>,
    pub char_width: Binding<f32>,
    pub pointer_pos: Binding<(f32, f32)>,
    /// rmb is right mouse button
    pub rmb_pressed: Binding<bool>,
    /// lmb is right mouse button
    pub lmb_pressed: Binding<bool>,
    pub editor_size: Binding<(f32, f32)>,
    pub drawing_pos: Binding<(f32, f32)>,

    pub cursor_instant: Binding<Instant>,

    pub selection_rectangle_radius: f32,

    pub braces: Vec<char>,
    pub matching_braces: Vec<char>,
}

pub const DISTANCE_BETWEEN_NUMBER_AND_LINE: f32 = 20.0;
pub const CURSOR_DURATION: u128 = 500;

impl EditorState {
    pub fn draw(&self, drawer: &Drawer, rect: Rect) {
        let monospace_font = &drawer.fonts[&MONOSPACE_FONT_ID];
        let canvas = drawer.canvas;
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

        // TODO
        self.line_height.set(font_height);
        self.char_width.set(font_width);
        self.drawing_pos.set((x1, y1));
        self.editor_size.set((x2 - x1, y2 - y1));

        let back_color = Color4f::new(39.0 / 255.0, 36.0 / 255.0, 52.0 / 255.0, 1.0);
        let front_color = Color4f::new(230.0 / 255.0, 230.0 / 255.0, 232.0 / 255.0, 1.0);

        canvas.draw_rect(&rect, &Paint::new(back_color, None));

        // Lines render
        // delta is distance between left-top corner of first displayed line and left-top corner of editor
        let delta_y = self.scroll.1 - font_height * (first_line as f32);
        let y1 = y1 - delta_y + font_height - first_line as f32 * font_height;
        for i in first_line..min(last_line, self.file.len()) {
            // Render line
            if !self.file[i].is_empty() {
                canvas.draw_text_blob(
                    TextBlob::new(&String::from_iter(&self.file[i]), monospace_font).unwrap(),
                    Point::new(x1, y1 + i as f32 * font_height),
                    &Paint::new(front_color, None),
                );
            }
            // Render number
            let linenum = format!("{}", i + 1);
            canvas.draw_text_blob(
                TextBlob::new(&linenum, monospace_font).unwrap(),
                Point::new(
                    x1 - DISTANCE_BETWEEN_NUMBER_AND_LINE - linenum.len() as f32 * font_width,
                    y1 + i as f32 * font_height,
                ),
                &Paint::new(front_color, None),
            );
        }
        let duration = Instant::now() - self.cursor_instant.get();
        let duration = duration.as_millis();
        if duration / CURSOR_DURATION % 2 == 0 {
            for cursor in &self.cursors {
                self.draw_cursor(
                    canvas,
                    cursor,
                    x1,
                    y1,
                    font_width,
                    font_height,
                    metrics.cap_height,
                );
            }
        }
        for cursor in &self.cursors {
            self.draw_selection(
                canvas,
                cursor,
                x1,
                x2,
                y1,
                font_width,
                font_height,
                metrics.cap_height,
            );
        }
        canvas.restore();
    }

    fn draw_cursor(
        &self,
        canvas: &Canvas,
        cursor: &Cursor,
        x1: f32,
        y1: f32,
        font_width: f32,
        font_height: f32,
        cap_height: f32,
    ) {
        let line = cursor.position.0;
        let ch = cursor.position.1;
        let x = x1 + ch as f32 * font_width;
        let y = y1 + line as f32 * font_height + (font_height - cap_height) / 2.0;
        canvas.draw_rect(
            &Rect::from_ltrb(x - 1.0, y - font_height, x + 1.0, y),
            &Paint::new(Color4f::new(0.0, 0.0, 1.0, 1.0), None),
        );
    }

    fn draw_selection(
        &self,
        canvas: &Canvas,
        cursor: &Cursor,
        x1: f32,
        x2: f32,
        y1: f32,
        font_width: f32,
        font_height: f32,
        cap_height: f32,
    ) {
        let selection_line = cursor.selection_pos.0;
        let selection_char = cursor.selection_pos.1;
        let line = cursor.position.0;
        let ch = cursor.position.1;
        let x = x1 + ch as f32 * font_width;
        let y = y1 + line as f32 * font_height + (font_height - cap_height) / 2.0;
        if selection_line == line {
            if ch != selection_char {
                let sel_x = x1 + selection_char as f32 * font_width;
                let (min_x, max_x) = if sel_x < x { (sel_x, x) } else { (x, sel_x) };
                canvas.draw_rrect(
                    &RRect::new_rect_radii(
                        &Rect::from_ltrb(min_x, y - font_height, max_x, y),
                        &[
                            (
                                self.selection_rectangle_radius,
                                self.selection_rectangle_radius,
                            )
                                .into(),
                            (
                                self.selection_rectangle_radius,
                                self.selection_rectangle_radius,
                            )
                                .into(),
                            (
                                self.selection_rectangle_radius,
                                self.selection_rectangle_radius,
                            )
                                .into(),
                            (
                                self.selection_rectangle_radius,
                                self.selection_rectangle_radius,
                            )
                                .into(),
                        ],
                    ),
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
                let y = y1 + min_span.0 as f32 * font_height + (font_height - cap_height) / 2.0;
                canvas.draw_rect(
                    &Rect::from_ltrb(x, y - font_height, x2, y),
                    &Paint::new(Color4f::new(0.0, 0.0, 1.0, 0.3), None),
                );
            }
            // max_span
            {
                let x = x1 + max_span.1 as f32 * font_width;
                let y = y1 + max_span.0 as f32 * font_height + (font_height - cap_height) / 2.0;
                canvas.draw_rect(
                    &Rect::from_ltrb(x1, y - font_height, x, y),
                    &Paint::new(Color4f::new(0.0, 0.0, 1.0, 0.3), None),
                );
            }
            // filled lines
            if max_span.0 - min_span.0 > 1 {
                let min_y = y1 + min_span.0 as f32 * font_height + (font_height - cap_height) / 2.0;
                let max_y =
                    y1 + (max_span.0 - 1) as f32 * font_height + (font_height - cap_height) / 2.0;
                canvas.draw_rect(
                    &Rect::from_ltrb(x1, min_y, x2, max_y),
                    &Paint::new(Color4f::new(0.0, 0.0, 1.0, 0.3), None),
                );
            }
        }
    }

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
        let mut cursor = self.cursors[cursor_id];
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
        self.cursor_sync_cords(cursor, cursor_id);
    }

    fn cursor_right(&mut self, cursor_id: usize) {
        let mut cursor = self.cursors[cursor_id];
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
        self.cursor_sync_cords(cursor, cursor_id);
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
            for i in 0..self.braces.len() {
                if ch == self.braces[i] {
                    let (min_span, max_span) = if cursor.selection_pos.0 < cursor.position.0 {
                        (&mut cursor.selection_pos, cursor.position)
                    } else {
                        (&mut cursor.position, cursor.selection_pos)
                    };
                    self.file[min_span.0].insert(min_span.1, ch);
                    self.file[max_span.0].insert(max_span.1, self.matching_braces[i]);
                    min_span.1 += 1;
                    self.cursors[cursor_id] = cursor;
                    return;
                }
            }
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
            for i in 0..self.braces.len() {
                if ch == self.braces[i] {
                    let (min_span, max_span) = if cursor.selection_pos.1 < cursor.position.1 {
                        (&mut cursor.selection_pos, &mut cursor.position)
                    } else {
                        (&mut cursor.position, &mut cursor.selection_pos)
                    };
                    self.file[min_span.0].insert(min_span.1, ch);
                    min_span.1 += 1;
                    max_span.1 += 1;
                    self.file[max_span.0].insert(max_span.1, self.matching_braces[i]);
                    self.cursors[cursor_id] = cursor;
                    return;
                }
            }

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

    fn cursor_handle_end(&mut self, cursor_id: usize) {
        let mut cursor = self.cursors[cursor_id];
        cursor.position.1 = self.file[cursor.position.0].len();
        self.cursor_sync_cords(cursor, cursor_id)
    }

    fn cursor_handle_end_select(&mut self, cursor_id: usize) {
        self.cursors[cursor_id].position.1 = self.file[self.cursors[cursor_id].position.0].len();
    }

    fn cursor_handle_start(&mut self, cursor_id: usize) {
        let mut cursor = self.cursors[cursor_id];
        let line = cursor.position.0;
        if cursor.position.1 == 0 {
            let mut index = 0;
            while index < self.file[line].len() && self.file[line][index] == ' ' {
                index += 1;
            }
            cursor.position.1 = index;
        } else {
            cursor.position.1 = 0;
        }
        self.cursor_sync_cords(cursor, cursor_id)
    }
    fn cursor_handle_start_select(&mut self, cursor_id: usize) {
        let cursor = &mut self.cursors[cursor_id];
        let line = cursor.position.0;
        if cursor.position.1 == 0 {
            let mut index = 0;
            while index < self.file[line].len() && self.file[line][index] == ' ' {
                index += 1;
            }
            cursor.position.1 = index;
        } else {
            cursor.position.1 = 0;
        }
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

    // fn handle_pg_up(&mut self, mut cursor: Cursor, cursor_id: usize) {}

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
                    if modifiers.state().super_key() {
                        if modifiers.state().shift_key() {
                            self.cursor_handle_start_select(cursor_id)
                        } else {
                            self.cursor_handle_start(cursor_id)
                        }
                    } else if modifiers.state().shift_key() {
                        self.cursor_selection_left(cursor_id)
                    } else {
                        self.cursor_left(cursor_id)
                    }
                }
                NamedKey::ArrowRight => {
                    if modifiers.state().super_key() {
                        if modifiers.state().shift_key() {
                            self.cursor_handle_end_select(cursor_id)
                        } else {
                            self.cursor_handle_end(cursor_id)
                        }
                    } else if modifiers.state().shift_key() {
                        self.cursor_selection_right(cursor_id)
                    } else {
                        self.cursor_right(cursor_id)
                    }
                }
                NamedKey::Tab => self.cursor_handle_tab(cursor_id),
                NamedKey::Delete => self.cursor_handle_delete(cursor_id),
                NamedKey::Enter => self.cursor_handle_return(cursor_id),
                NamedKey::Backspace => self.cursor_handle_backspace(cursor_id),
                NamedKey::End => self.cursor_handle_end(cursor_id),
                NamedKey::Home => self.cursor_handle_start(cursor_id),
                NamedKey::Space => self.cursor_handle_char(cursor_id, ' '),
                _ => {}
            },
            Key::Character(ref ch) => {
                let v = ch.chars().collect::<Vec<char>>();
                self.cursor_handle_char(cursor_id, v[0])
            }
            _ => {}
        }
        for cur in 0..self.cursors.len() {
            self.caret_scroll(cur);
        }
    }

    pub fn handle_ime(&mut self, cursor_id: usize, ime: &Ime) {
        let mut cursor = self.cursors[cursor_id];
        match ime {
            Ime::Preedit(s, len) => {
                for _ in 0..cursor.last_ime_len {
                    self.cursor_handle_backspace(cursor_id);
                }

                match len {
                    Some((a, _)) => {
                        cursor.last_ime_len = *a;
                        let chars = s.chars().collect::<Vec<char>>();
                        for ch in chars {
                            self.cursor_handle_char(cursor_id, ch);
                        }
                    }
                    _ => {
                        cursor.last_ime_len = 0;
                    }
                }
            }
            Ime::Commit(s) => {
                let chars = s.chars().collect::<Vec<char>>();
                for ch in &chars {
                    self.cursor_handle_char(cursor_id, ch.clone());
                }
                cursor.last_ime_len = 0;
            }
            _ => {}
        }

        self.cursor_sync_cords(cursor, cursor_id);
        for cur in 0..self.cursors.len() {
            self.caret_scroll(cur);
        }
    }

    pub fn handle_scroll(&mut self, delta: (f32, f32)) {
        self.scroll.1 += delta.1;

        if self.scroll.1 < 0.0 {
            self.scroll.1 = 0.0;
        } else {
            if self.file.len() > 1 {
                let self_height = (self.file.len() as f32 - 1.0) * self.line_height.get();
                if self.scroll.1 > self_height {
                    self.scroll.1 = self_height;
                }
            } else {
                self.scroll.1 = 0.0;
            }
        }
    }

    pub fn handle_left_mouse_press(&mut self, modifiers: &Modifiers) {
        if !modifiers.state().shift_key() {
            let pos = self.pointer_pos.get();
            let size = self.editor_size.get();
            if pos.0 > 0.0 && pos.1 > 0.0 && pos.0 < size.0 && pos.1 < size.1 {
                let line = (pos.1 + self.scroll.1) / self.line_height.get();
                let character = (pos.0 + self.scroll.0) / self.char_width.get();
                let line = if line < 0.0 {
                    0
                } else if line as usize >= self.file.len() {
                    self.file.len() - 1
                } else {
                    line as usize
                };
                let character = if character < 0.0 {
                    0
                } else if character as usize > self.file[line].len() {
                    self.file[line].len()
                } else {
                    character as usize
                };
                self.cursors = vec![Cursor {
                    selection_pos: (line, character),
                    position: (line, character),
                    normal_x: character,
                    last_ime_len: 0,
                }];
            }
        }
        self.lmb_pressed.set(true);
        for cur in 0..self.cursors.len() {
            self.caret_scroll(cur);
        }
    }

    pub fn handle_left_mouse_release(&mut self, _modifiers: &Modifiers) {
        self.lmb_pressed.set(false);
        for cur in 0..self.cursors.len() {
            self.caret_scroll(cur);
        }
    }

    pub fn handle_mouse_movement(&mut self, point: (f32, f32)) {
        let editor_position = self.drawing_pos.get();
        self.pointer_pos
            .set((point.0 - editor_position.0, point.1 - editor_position.1));
        if self.lmb_pressed.get() {
            let pos = self.pointer_pos.get();
            let line = (pos.1 + self.scroll.1) / self.line_height.get();
            let character = (pos.0 + self.scroll.0) / self.char_width.get();
            let line = if line < 0.0 {
                0
            } else if line as usize >= self.file.len() {
                self.file.len() - 1
            } else {
                line as usize
            };
            let character = if character < 0.0 {
                0
            } else if character as usize > self.file[line].len() {
                self.file[line].len()
            } else {
                character as usize
            };

            self.cursors[0].position = (line, character);
            self.cursors[0].normal_x = character;
            for cur in 0..self.cursors.len() {
                self.caret_scroll(cur);
            }
        }
    }

    pub fn sync_cursor_time(&mut self) {
        self.cursor_instant.set(Instant::now())
    }

    fn caret_scroll(&mut self, cursor: usize) {
        let pos = self.cursors[cursor].position;
        let line_height = self.line_height.get();
        let size = self.editor_size.get();
        if self.file.len() + 4 < (size.1 / line_height) as usize {
            self.scroll.1 = 0.0;
        } else {
            if line_height * (self.file.len() - 4) as f32 - self.scroll.1 < 0.0 {
                self.scroll.1 = {
                    let tmp = line_height * (self.file.len() as f32 - 4.0) - size.1;
                    if tmp > 0.0 {
                        tmp
                    } else {
                        0.0
                    }
                };
            }
            let caret_pixel_pos = pos.0 as f32 * line_height - self.scroll.1;
            if caret_pixel_pos < line_height * 2.0 {
                self.scroll.1 -= line_height * 2.0 - caret_pixel_pos;
            } else if caret_pixel_pos > size.1 - line_height * 2.0 {
                self.scroll.1 += caret_pixel_pos - size.1 + line_height * 2.0;
            }
            if self.scroll.1 < 0.0 {
                self.scroll.1 = 0.0;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum SpanType {
    // Text,
    // Comment,
    // Keyword,
}
