use super::*;

use shell::{
    kurbo::Size,
    piet::{Piet, RenderContext},
    MouseButton,
};

pub struct Clickable<F: Fn(&mut Context, MouseButton)> {
    child: ViewId,
    on_click: F,
}

impl<F: Fn(&mut Context, MouseButton)> Clickable<F> {
    pub fn new(child: ViewId, on_click: F) -> Self {
        Self { child, on_click }
    }

    fn update_layout(&self, id: ViewId, drawer: &mut Piet, max_size: Size, ctx: &mut Context) {
        let offset = drawer.current_transform();
        ctx.set_layout(id, Layout::new(offset, max_size));
    }
}

impl<F: Fn(&mut Context, MouseButton)> View for Clickable<F> {
    fn draw(&self, draw_ctx: DrawContext<'_, '_, '_>) {
        let DrawContext {
            id,
            drawer,
            size: max_size,
            ctx,
        } = draw_ctx;
        self.update_layout(id, drawer, max_size, ctx);
        ctx.map_view(self.child, &mut |view, ctx| {
            view.draw(DrawContext {
                id: self.child,
                drawer,
                size: max_size,
                ctx,
            })
        })
    }

    fn process_event(&mut self, event: &Event, ctx: &mut Context) -> bool {
        match event {
            Event::MousePress(button) => {
                (self.on_click)(ctx, *button);
                true
            }
            Event::MouseUnpress(_) => {
                println!("unpress");
                true
            }
            _ => false,
        }
    }

    fn get_min_size(&self, drawer: &mut Piet, ctx: &mut Context) -> Size {
        ctx.map_view(self.child, &mut |view, ctx| view.get_min_size(drawer, ctx))
    }

    fn is_flexible(&self) -> bool {
        true
    }
}
