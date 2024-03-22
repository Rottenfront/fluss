use super::*;
use shell::{
    kurbo::{Rect, Size},
    piet::{Color, Piet, RenderContext},
};
pub struct Filler {
    color: fn() -> Color,
}

impl Filler {
    pub fn new(color: fn() -> Color) -> Self {
        Self { color }
    }
}

impl View for Filler {
    fn draw(&self, id: ViewId, drawer: &mut Piet, max_size: Size, ctx: &mut Context) {
        let offset = drawer.current_transform();
        ctx.set_layout(id, Layout::new(offset, max_size));
        let color = (self.color)();
        drawer.fill(&Rect::from_origin_size((0.0, 0.0), max_size), &color);
    }

    fn process_event(&mut self, _event: &Event, _ctx: &mut Context, _drawer: &mut Piet) -> bool {
        false
    }

    fn get_min_size(&self, _drawer: &mut Piet) -> Size {
        Size::default()
    }

    fn is_flexible(&self) -> bool {
        true
    }
}
