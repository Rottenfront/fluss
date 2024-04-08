use crate::*;

pub struct Clickable<V: View, F: Fn(&mut Context, MouseButton, Point)> {
    id: ViewId,
    child: V,
    on_click: F,
}

impl<V: View, F: Fn(&mut Context, MouseButton, Point)> Clickable<V, F> {
    pub fn new(child: V, on_click: F) -> Self {
        Self {
            id: new_id(),
            child,
            on_click,
        }
    }
}

impl<V: View, F: Fn(&mut Context, MouseButton, Point)> View for Clickable<V, F> {
    fn draw(&self, draw_ctx: DrawContext<'_, '_>) {
        let DrawContext {
            drawer,
            size: max_size,
            ctx,
        } = draw_ctx;
        self.update_layout(Layout::new(drawer.current_transform(), max_size), ctx);
        self.child.update_parent(self.get_id(), ctx);
        self.child.draw(DrawContext {
            drawer,
            size: max_size,
            ctx,
        })
    }

    fn get_id(&self) -> ViewId {
        self.id
    }

    fn get_min_size(&self, ctx: &mut Context) -> Size {
        self.child.get_min_size(ctx)
    }

    fn is_flexible(&self) -> bool {
        self.child.is_flexible()
    }

    fn update(&mut self, ctx: &mut Context) {
        self.child.update(ctx)
    }

    fn mouse_press(&mut self, event: &MousePress, ctx: &mut Context) -> bool {
        (self.on_click)(ctx, event.button, event.pos);
        true
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
