use super::*;
use shell::{kurbo::Size, piet::Piet};

pub trait View {
    fn draw(&self, draw_ctx: DrawContext);

    /// true if processed
    fn process_event(&mut self, event: &Event, ctx: &mut Context) -> bool;

    fn get_min_size(&self, drawer: &mut Piet, ctx: &mut Context) -> Size;

    fn is_flexible(&self) -> bool;
}
