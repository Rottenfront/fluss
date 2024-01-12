mod backend;
mod public_api;

use backend::{SkiaBackend, SkiaEnv};
use binding::{bind, Binding, Bound, MutableBound};
use public_api::Widget;

use std::{
    cmp::min,
    collections::HashMap,
    path::PathBuf,
    time::{Duration, Instant},
};

use skia_safe::{
    font_style::{Slant, Weight, Width},
    Canvas, ClipOp, Color, Color4f, Font, FontMgr, FontStyle, Paint, Point, RRect, Rect, Size,
    TextBlob,
};
use winit::{
    dpi::LogicalSize,
    event::{
        Event, Ime, KeyEvent, Modifiers, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey, PhysicalKey},
    window::WindowBuilder,
};

enum AppFocus {
    LeftDock,
    RightDock,
    BottomDock,
    Editors,
    None,
}

struct TabSystem {
    scroll: f32,
    // change EditorState to trait TabState
    states: Vec<EditorState>,
    enabled: usize,
}

enum SplitDirection {
    Vertical,
    Horizontal,
}

enum SplitItem {
    Split(Split),
    TabSystem(TabSystem),
}

impl SplitItem {
    pub fn draw(&self, canvas: &Canvas, rect: Rect, monospace_font: &Font) {
        match self {
            Self::Split(sp) => sp.draw(canvas, rect, monospace_font),
            Self::TabSystem(ts) => ts.draw(canvas, rect, monospace_font),
        }
    }
}

impl TabSystem {
    pub fn draw(&self, canvas: &Canvas, rect: Rect, monospace_font: &Font) {
        let (x1, y1, x2, y2) = (rect.left, rect.top, rect.right, rect.bottom);
        let row_color = Color4f::new(61.0 / 255.0, 61.0 / 255.0, 61.0 / 255.0, 1.0);
        canvas.draw_rect(
            &Rect::from_ltrb(x1, y1, x2, y1 + UPPER_BAR_HEIGHT),
            &Paint::new(row_color, None),
        );
        self.states[self.enabled].draw(
            canvas,
            Rect {
                left: x1,
                top: y1 + UPPER_BAR_HEIGHT,
                right: x2,
                bottom: y2,
            },
            monospace_font,
        );
    }

    pub fn focused_editor_mut(&mut self) -> &mut EditorState {
        &mut self.states[self.enabled]
    }
    pub fn focused_editor(&self) -> &EditorState {
        &self.states[self.enabled]
    }
}

struct Split {
    direction: SplitDirection,
    main_item: Box<SplitItem>,
    next_item: Option<Box<SplitItem>>,
    // between 0 and 1 - percentage of main item width or height depending on the direction
    fraction: f32,
    // true is enabled next item, false if enabled main item
    enabled: bool,
}

impl Split {
    pub fn focused_tab_system_mut(&mut self) -> &mut TabSystem {
        match &mut self.next_item {
            None => match &mut *self.main_item {
                SplitItem::Split(split) => split.focused_tab_system_mut(),
                SplitItem::TabSystem(ts) => ts,
            },
            Some(sp) => {
                if self.enabled {
                    match &mut **sp {
                        SplitItem::Split(split) => split.focused_tab_system_mut(),
                        SplitItem::TabSystem(ts) => ts,
                    }
                } else {
                    match &mut *self.main_item {
                        SplitItem::Split(split) => split.focused_tab_system_mut(),
                        SplitItem::TabSystem(ts) => ts,
                    }
                }
            }
        }
    }
    pub fn focused_tab_system(&self) -> &TabSystem {
        match &self.next_item {
            None => match &*self.main_item {
                SplitItem::Split(split) => split.focused_tab_system(),
                SplitItem::TabSystem(ts) => ts,
            },
            Some(sp) => {
                if self.enabled {
                    match &**sp {
                        SplitItem::Split(split) => split.focused_tab_system(),
                        SplitItem::TabSystem(ts) => ts,
                    }
                } else {
                    match &*self.main_item {
                        SplitItem::Split(split) => split.focused_tab_system(),
                        SplitItem::TabSystem(ts) => ts,
                    }
                }
            }
        }
    }

    pub fn draw(&self, canvas: &Canvas, rect: Rect, monospace_font: &Font) {
        let (x1, y1, x2, y2) = (rect.left, rect.top, rect.right, rect.bottom);
        match &self.next_item {
            None => self.main_item.draw(canvas, rect, monospace_font),
            Some(next) => match self.direction {
                SplitDirection::Vertical => {
                    let middle_y = y1 * self.fraction + y2 * (1.0 - self.fraction);
                    self.main_item.draw(
                        canvas,
                        Rect {
                            left: x1,
                            top: y1,
                            right: x2,
                            bottom: middle_y,
                        },
                        monospace_font,
                    );
                    next.draw(
                        canvas,
                        Rect {
                            left: x1,
                            top: middle_y,
                            right: x2,
                            bottom: y2,
                        },
                        monospace_font,
                    );
                }
                SplitDirection::Horizontal => {
                    let middle_x = x1 * self.fraction + x2 * (1.0 - self.fraction);
                    self.main_item.draw(
                        canvas,
                        Rect {
                            left: x1,
                            top: y1,
                            right: middle_x,
                            bottom: y2,
                        },
                        monospace_font,
                    );
                    next.draw(
                        canvas,
                        Rect {
                            left: middle_x,
                            top: y1,
                            right: x2,
                            bottom: y2,
                        },
                        monospace_font,
                    );
                }
            },
        }
    }
}

struct ApplicationState {
    monospace_font: Font,
    editors: Split,
    focus: AppFocus,
}

struct DockState {
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
    last_ime_len: usize,
}

pub struct EditorState {
    file: Vec<Vec<char>>,
    name: String,
    path: Option<PathBuf>,
    scroll: (f32, f32),
    // spans: Binding<Vec<(usize, usize, usize, usize, SpanType)>>,
    cursors: Vec<Cursor>,
    tab_length: usize,
    line_height: Binding<f32>,
    char_width: Binding<f32>,
    pointer_pos: Binding<(f32, f32)>,
    /// rmb is right mouse button
    rmb_pressed: Binding<bool>,
    /// lmb is right mouse button
    lmb_pressed: Binding<bool>,
    editor_size: Binding<(f32, f32)>,
    drawing_pos: Binding<(f32, f32)>,

    selection_rectangle_radius: f32,
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
            let linenum = format!("{i}");
            canvas.draw_text_blob(
                TextBlob::new(&linenum, monospace_font).unwrap(),
                Point::new(
                    x1 - DISTANCE_BETWEEN_NUMBER_AND_LINE - linenum.len() as f32 * font_width,
                    y1 + i as f32 * font_height,
                ),
                &Paint::new(front_color, None),
            );
        }
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

    fn cursor_handle_end(&mut self, cursor_id: usize) {
        let mut cursor = self.cursors[cursor_id];
        cursor.position.1 = self.file[cursor.position.0].len();
        self.cursor_sync_cords(cursor, cursor_id)
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
                    if modifiers.state().super_key() {
                        self.cursor_handle_start(cursor_id)
                    } else if modifiers.state().shift_key() {
                        self.cursor_selection_left(cursor_id)
                    } else {
                        self.cursor_left(cursor_id)
                    }
                }
                NamedKey::ArrowRight => {
                    if modifiers.state().super_key() {
                        self.cursor_handle_end(cursor_id)
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
    }

    pub fn handle_ime(&mut self, cursor_id: usize, ime: &Ime) {
        let mut cursor = self.cursors[cursor_id];
        match ime {
            Ime::Preedit(s, len) => {
                for _ in 0..cursor.last_ime_len {
                    self.cursor_handle_backspace(cursor_id);
                }

                match len {
                    Some((a, b)) => {
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
    }

    pub fn handle_scroll(&mut self, delta: (f32, f32)) {
        self.scroll.1 += delta.1;

        if self.scroll.1 < 0.0 {
            self.scroll.1 = 0.0;
        } else if self.file.len() > 1 {
            let self_height = (self.file.len() as f32 - 1.0) * self.line_height.get();
            if self.scroll.1 > self_height {
                self.scroll.1 = self_height;
            }
        }
    }

    pub fn handle_left_mouse_press(&mut self) {
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

            self.lmb_pressed.set(true);
        }
    }

    pub fn handle_left_mouse_release(&mut self) {
        self.lmb_pressed.set(false);
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
        }
    }
}

impl ApplicationState {
    /// Renders a rectangle that occupies exactly half of the canvas
    fn draw(&self, canvas: &Canvas) {
        let canvas_size = Size::from(canvas.base_layer_size());

        canvas.clear(Color::WHITE);

        self.editors.draw(
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
        editors: Split {
            direction: SplitDirection::Horizontal,
            main_item: Box::new(SplitItem::TabSystem(TabSystem {
                scroll: 0.0,
                states: vec![EditorState {
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
                        last_ime_len: 0,
                    }],
                    line_height: bind(14.0),
                    char_width: bind(2.0),
                    pointer_pos: bind((0.0, 0.0)),
                    lmb_pressed: bind(false),
                    rmb_pressed: bind(false),
                    editor_size: bind((0.0, 0.0)),
                    drawing_pos: bind((0.0, 0.0)),
                    tab_length: 4,
                    selection_rectangle_radius: 3.0,
                }],
                enabled: 0,
            })),
            next_item: None,
            fraction: 1.0,
            enabled: false,
        },
        focus: AppFocus::Editors,
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
        .with_title("Fluss")
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
    let mut mouse_map: HashMap<MouseButton, bool> = HashMap::new();

    env.window().set_ime_allowed(true);
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
                WindowEvent::MouseWheel { delta, phase, .. } => match phase {
                    TouchPhase::Started => {}
                    TouchPhase::Moved => app
                        .editors
                        .focused_tab_system_mut()
                        .focused_editor_mut()
                        .handle_scroll(match delta {
                            MouseScrollDelta::LineDelta(x, y) => (x, y),
                            MouseScrollDelta::PixelDelta(pos) => (pos.x as _, -pos.y as _),
                        }),
                    TouchPhase::Ended => {}
                    _ => {}
                },
                WindowEvent::ModifiersChanged(new_modifiers) => modifiers = new_modifiers,
                WindowEvent::Ime(ime) => {
                    for id in 0..app
                        .editors
                        .focused_tab_system_mut()
                        .focused_editor_mut()
                        .cursors
                        .len()
                    {
                        app.editors
                            .focused_tab_system_mut()
                            .focused_editor_mut()
                            .handle_ime(id, &ime)
                    }
                }
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
                        // if !modifiers.state().super_key() && !modifiers.state().control_key() {
                        for id in 0..app
                            .editors
                            .focused_tab_system_mut()
                            .focused_editor_mut()
                            .cursors
                            .len()
                        {
                            app.editors
                                .focused_tab_system_mut()
                                .focused_editor_mut()
                                .handle_cursor_input(id, logical_key.clone(), &modifiers);
                        }
                        // }
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
                WindowEvent::CursorMoved { position, .. } => {
                    app.editors
                        .focused_tab_system_mut()
                        .focused_editor_mut()
                        .handle_mouse_movement((position.x as _, position.y as _));
                }
                WindowEvent::MouseInput { button, .. } => {
                    if !mouse_map.contains_key(&button) {
                        mouse_map.insert(button, false);
                    }
                    if !mouse_map[&button] {
                        match button {
                            MouseButton::Left => {
                                app.editors
                                    .focused_tab_system_mut()
                                    .focused_editor_mut()
                                    .handle_left_mouse_press();
                            }
                            _ => {}
                        }
                    } else {
                        match button {
                            MouseButton::Left => {
                                app.editors
                                    .focused_tab_system_mut()
                                    .focused_editor_mut()
                                    .handle_left_mouse_release();
                            }
                            _ => {}
                        }
                    }
                    let b = mouse_map.remove(&button).unwrap();
                    mouse_map.insert(button, !b);
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
