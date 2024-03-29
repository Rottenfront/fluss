use super::*;
use shell::{kurbo::Size, piet::Piet};

pub trait View {
    fn draw(&self, draw_ctx: DrawContext);

    fn get_id(&self) -> ViewId;

    fn update_layout(&self, layout: Layout, ctx: &mut Context) {
        ctx.set_layout(self.get_id(), layout);
    }

    fn get_layout(&self, ctx: &mut Context) -> Option<Layout> {
        ctx.get_layout(self.get_id()).map(|layout| layout)
    }

    fn update_parent(&self, parent: ViewId, ctx: &mut Context) {
        ctx.set_parent_view(self.get_id(), parent);
    }

    fn get_parent(&self, ctx: &mut Context) -> Option<ViewId> {
        ctx.get_parent_view(self.get_id())
    }

    /// true if processed
    fn process_event(&mut self, event: &Event, ctx: &mut Context) -> bool;

    fn get_min_size(&self, drawer: &mut Piet, ctx: &mut Context) -> Size;

    fn is_flexible(&self) -> bool;
}
