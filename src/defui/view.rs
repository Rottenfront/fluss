use super::*;
use shell::{kurbo::Size, piet::Piet};

pub trait View {
    fn draw(&self, id: ViewId, drawer: &mut Piet, max_size: Size, ctx: &mut Context);

    /// true if processed
    fn process_event(&mut self, event: &Event, ctx: &mut Context, drawer: &mut Piet) -> bool;

    fn get_min_size(&self, drawer: &mut Piet) -> Size;

    fn is_flexible(&self) -> bool;
}
