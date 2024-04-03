use super::*;

use shell::{
    kurbo::{Point, Size},
    piet::{Piet, RenderContext},
    MouseButton,
};

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
    fn draw(&self, draw_ctx: DrawContext<'_, '_, '_>) {
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

    fn process_event(&mut self, event: &Event, ctx: &mut Context) -> bool {
        match event {
            Event::MousePress { button, pos } => {
                (self.on_click)(ctx, *button, *pos);
                true
            }
            Event::MouseUnpress { .. } => {
                println!("unpress");
                true
            }
            _ => false,
        }
    }

    fn get_min_size(&self, drawer: &mut Piet, ctx: &mut Context) -> Size {
        self.child.get_min_size(drawer, ctx)
    }

    fn is_flexible(&self) -> bool {
        self.child.is_flexible()
    }
}
