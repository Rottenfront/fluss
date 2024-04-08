use crate::*;
pub struct Filler {
    id: ViewId,
    color: fn() -> Color,
}

impl Filler {
    pub fn new(color: fn() -> Color) -> Self {
        Self {
            id: new_id(),
            color,
        }
    }
}

impl View for Filler {
    fn draw(&self, draw_ctx: DrawContext) {
        let DrawContext {
            drawer,
            size: max_size,
            ctx,
        } = draw_ctx;
        let offset = drawer.current_transform();
        ctx.set_layout(self.id, Layout::new(offset, max_size));
        let color = (self.color)();
        drawer.fill_rect(RectQuad::new(Point::ZERO, max_size).with_color(color));
    }

    fn get_id(&self) -> ViewId {
        self.id
    }

    fn get_min_size(&self, _ctx: &mut Context) -> Size {
        Size::default()
    }

    fn is_flexible(&self) -> bool {
        true
    }

    fn mouse_press(&mut self, event: &MousePress, ctx: &mut Context) -> bool {
        false
    }

    fn mouse_unpress(&mut self, event: &MouseUnpress, ctx: &mut Context) -> bool {
        false
    }

    fn mouse_focus_lost(&mut self, ctx: &mut Context) -> bool {
        false
    }

    fn mouse_focus_gained(&mut self, ctx: &mut Context) -> bool {
        false
    }

    fn scroll(&mut self, event: &ScrollEvent, ctx: &mut Context) -> bool {
        false
    }

    fn keyboard_focus_lost(&mut self, ctx: &mut Context) -> bool {
        false
    }

    fn keyboard_focus_gained(&mut self, ctx: &mut Context) -> bool {
        false
    }

    fn keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut Context) -> bool {
        false
    }

    fn input_method(&mut self, event: &ImeEvent, ctx: &mut Context) -> bool {
        false
    }

    fn mouse_move(&mut self, relative_pos: &Point, ctx: &mut Context) -> bool {
        false
    }

    fn is_scrollable(&self) -> bool {
        false
    }

    fn has_ime(&self) -> bool {
        false
    }
}
