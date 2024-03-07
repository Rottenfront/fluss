pub use trist::*;

pub struct Context {}

pub enum Event {}

pub trait View {
    fn draw(&self, drawer: &mut Drawer, transform: Affine, ctx: &Context);

    /// true if processed
    fn process_event(
        &mut self,
        event: &Event,
        draw_ctx: &mut DrawerState,
        ctx: &mut Context,
    ) -> bool;

    fn get_min_size(&self, draw_ctx: &DrawerState) -> Size;

    fn is_flexible(&self) -> bool;
}
