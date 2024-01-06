use futures::{executor, prelude::*};
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

use crate::drawer::Drawer;

pub mod highlight_thread;
pub mod lsp_thread;

pub struct HoverWindowState {
    enabled: Binding<bool>,
    position: Binding<(usize, usize)>,
    path: Binding<Option<String>>,
    markdown: Binding<String>,
    scroll: Binding<f32>,
    bounds: Binding<(f32, f32)>,
}

pub enum CompletionType {
    Function,
    Method,
    Interface,
    Class,
    Keyword,
    Module,
}

pub struct CompletionElement {
    type_: CompletionType,
    name: String,
    comment: Option<String>,
    action: usize,
    description: Option<String>,
}

pub struct CompletionWindowState {
    enabled: Binding<bool>,
    position: Binding<(usize, usize)>,
    elements: Binding<Vec<CompletionElement>>,
    selected: Binding<usize>,
    scroll_description: Binding<f32>,
    bounds: Binding<(f32, f32)>,
}

pub struct ActionWindowState {
    enabled: Binding<bool>,
    position: Binding<(usize, usize)>,
    elements: Binding<Vec<CompletionElement>>,
}

pub enum ErrorType {
    Warning,
    Error,
}

pub struct DiagnosticsWindowState {
    enabled: Binding<bool>,
    position: Binding<(usize, usize)>,
    content: Binding<String>,
    type_: Binding<ErrorType>,
}

pub struct FindWindowState {
    enabled: Binding<bool>,
    content: Binding<String>,
    is_regex: Binding<bool>,
    is_case_matching: Binding<bool>,
    replace_enabled: Binding<bool>,
    replace: Binding<String>,
    number_enabled: Binding<usize>,
}

#[derive(Debug, Clone)]
pub enum SpanType {
    Text,
    Comment,
    Keyword,
}

pub struct EditorState {
    hover: HoverWindowState,
    completion: CompletionWindowState,
    action_window: ActionWindowState,
    diagnostics: DiagnosticsWindowState,
    finder: FindWindowState,

    file: Binding<Vec<String>>,
    name: Binding<String>,
    path: Binding<Option<PathBuf>>,
    scroll: Binding<(f32, f32)>,
    spans: Binding<Vec<(usize, usize, usize, usize, SpanType)>>,
    cursors: Binding<Vec<(usize, usize, usize, usize)>>,
}

const DISTANCE_BETWEEN_NUMBER_AND_LINE: f32 = 20.0;

impl EditorState {
    pub fn draw(
        &self,
        gc: &mut Drawer,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        metrics: &FontMetrics,
        monospace_width: f32,
    ) {
        let scroll = self.scroll.get();
        let first_line = (scroll.1 / metrics.height) as usize;
        let last_line = ((scroll.1 + (y2 - y1)) / metrics.height) as usize + 1;
        // println!("{first_line}, {last_line}");
        // gc.unclip();
        gc.new_path();
        gc.rect(x1, y1, x2, y2);
        gc.clip();

        let x1 = DISTANCE_BETWEEN_NUMBER_AND_LINE + x1;

        // Lines render
        // delta is distance between left-top corner of first displayed line and left-top corner of editor
        let delta_y = scroll.1 - metrics.height * (first_line as f32);
        let y1 = y1 - delta_y + (1 - first_line) as f32 * metrics.height;
        gc.fill_color(Color::Rgba(0.0, 0.0, 0.0, 1.0));
        let lines = self.file.get();
        for i in first_line..min(last_line, lines.len()) {
            // Render line
            gc.begin_line_layout(x1, y1 + i as f32 * metrics.height, TextAlignment::Left);
            gc.layout_text(crate::MONOSPACE_FONT, lines[i].clone());
            gc.draw_text_layout();
        }

        gc.unclip();
    }

    pub fn draw_cursors(
        &self,
        gc: &mut Drawer,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        metrics: &FontMetrics,
    ) {
    }
}
