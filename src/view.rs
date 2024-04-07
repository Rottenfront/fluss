use super::*;

pub trait View {
    fn draw(&self, draw_ctx: DrawContext);

    fn get_id(&self) -> ViewId;

    /// true if processed
    fn mouse_press(&mut self, event: &MousePress, ctx: &mut Context) -> bool;

    /// true if processed
    fn mouse_unpress(&mut self, event: &MouseUnpress, ctx: &mut Context) -> bool;

    /// true if processed
    fn mouse_focus_lost(&mut self, ctx: &mut Context) -> bool;

    /// true if processed
    fn mouse_focus_gained(&mut self, ctx: &mut Context) -> bool;

    /// true if processed
    fn scroll(&mut self, event: &ScrollEvent, ctx: &mut Context) -> bool;

    /// true if processed
    fn keyboard_focus_lost(&mut self, ctx: &mut Context) -> bool;

    /// true if processed
    fn keyboard_focus_gained(&mut self, ctx: &mut Context) -> bool;

    /// true if processed
    fn keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut Context) -> bool;

    /// true if processed
    fn input_method(&mut self, event: &ImeEvent, ctx: &mut Context) -> bool;
    
    fn mouse_move(&mut self, relative_pos: &Point, ctx: &mut Context) -> bool;
    
    fn update(&mut self, ctx: &mut Context) {}
    
    fn get_min_size(&self, drawer: &mut Renderer, ctx: &mut Context) -> Size;

    fn is_flexible(&self) -> bool;
    
    fn is_scrollable(&self) -> bool;
    
    fn has_ime(&self) -> bool;
}

pub trait ViewHelpers: View {
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
}

impl<V: ?Sized + View> ViewHelpers for V {}
