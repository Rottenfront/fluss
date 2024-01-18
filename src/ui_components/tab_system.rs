use crate::canvas::Drawer;

use super::editor::EditorState;

use skia_safe::{Canvas, Color4f, Font, Paint, Rect};
pub struct TabSystem {
    pub scroll: f32,
    // change EditorState to trait TabState
    pub states: Vec<EditorState>,
    pub enabled: usize,
}

pub const TAB_BAR_HEIGHT: f32 = 30.0;

impl TabSystem {
    pub fn draw(&self, drawer: &Drawer, rect: Rect) {
        let (x1, y1, x2, y2) = (rect.left, rect.top, rect.right, rect.bottom);
        let row_color = Color4f::new(61.0 / 255.0, 61.0 / 255.0, 61.0 / 255.0, 1.0);
        drawer.canvas.draw_rect(
            &Rect::from_ltrb(x1, y1, x2, y1 + TAB_BAR_HEIGHT),
            &Paint::new(row_color, None),
        );
        self.states[self.enabled].draw(
            drawer,
            Rect {
                left: x1,
                top: y1 + TAB_BAR_HEIGHT,
                right: x2,
                bottom: y2,
            },
        );
    }

    pub fn focused_editor_mut(&mut self) -> &mut EditorState {
        &mut self.states[self.enabled]
    }
    pub fn focused_editor(&self) -> &EditorState {
        &self.states[self.enabled]
    }
}
