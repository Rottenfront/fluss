use super::*;
use shell::{
    kurbo::{Rect, Size},
    piet::{Color, Piet, RenderContext},
};
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
        drawer.fill(&Rect::from_origin_size((0.0, 0.0), max_size), &color);
    }

    fn get_id(&self) -> ViewId {
        self.id
    }

    fn process_event(&mut self, _event: &Event, _ctx: &mut Context) -> bool {
        false
    }

    fn get_min_size(&self, _drawer: &mut Piet, _ctx: &mut Context) -> Size {
        Size::default()
    }

    fn is_flexible(&self) -> bool {
        true
    }
}
