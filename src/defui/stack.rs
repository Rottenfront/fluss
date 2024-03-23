use super::*;
use shell::{
    kurbo::{Affine, Size},
    piet::{Piet, RenderContext},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackDirection {
    Vertical,
    Horizontal,
    Depth,
}

pub struct Stack {
    direction: StackDirection,
    views: Vec<ViewId>,
}

impl Stack {
    pub fn vstack(views: Vec<ViewId>) -> Self {
        Self {
            direction: StackDirection::Vertical,
            views,
        }
    }

    pub fn hstack(views: Vec<ViewId>) -> Self {
        Self {
            direction: StackDirection::Horizontal,
            views,
        }
    }

    pub fn zstack(views: Vec<ViewId>) -> Self {
        Self {
            direction: StackDirection::Depth,
            views,
        }
    }

    fn update_layout(&self, id: ViewId, drawer: &mut Piet, max_size: Size, ctx: &mut Context) {
        let offset = drawer.current_transform();
        ctx.set_layout(id, Layout::new(offset, max_size));
    }

    fn draw_vertical(&self, self_id: ViewId, drawer: &mut Piet, max_size: Size, ctx: &mut Context) {
        let height = max_size.height / (self.views.len() as f64);
        let mut current_offset = 0.0;
        for id in &self.views {
            ctx.set_parent_view(*id, self_id);
            ctx.map_view(*id, &mut |view, ctx| {
                let _ = drawer.save();
                drawer.transform(Affine::translate((0.0, current_offset)));
                view.draw(DrawContext {
                    id: *id,
                    drawer,
                    size: Size::new(max_size.width, height),
                    ctx,
                });
                current_offset += height;
                let _ = drawer.restore();
            });
        }
    }

    fn draw_horizontal(
        &self,
        self_id: ViewId,
        drawer: &mut Piet,
        max_size: Size,
        ctx: &mut Context,
    ) {
        let width = max_size.width / (self.views.len() as f64);
        let mut current_offset = 0.0;
        for id in &self.views {
            ctx.set_parent_view(*id, self_id);
            ctx.map_view(*id, &mut |view, ctx| {
                let _ = drawer.save();
                drawer.transform(Affine::translate((current_offset, 0.0)));
                view.draw(DrawContext {
                    id: *id,
                    drawer,
                    size: Size::new(width, max_size.height),
                    ctx,
                });
                current_offset += width;

                let _ = drawer.restore();
            });
        }
    }

    fn draw_depth(&self, self_id: ViewId, drawer: &mut Piet, max_size: Size, ctx: &mut Context) {
        for id in &self.views {
            ctx.map_view(*id, &mut |view, ctx| {
                ctx.set_parent_view(*id, self_id);
                view.draw(DrawContext {
                    id: *id,
                    drawer,
                    size: max_size,
                    ctx,
                });
            });
        }
    }

    fn draw_views(&self, self_id: ViewId, drawer: &mut Piet, max_size: Size, ctx: &mut Context) {
        match self.direction {
            StackDirection::Vertical => self.draw_vertical(self_id, drawer, max_size, ctx),
            StackDirection::Horizontal => self.draw_horizontal(self_id, drawer, max_size, ctx),
            StackDirection::Depth => self.draw_depth(self_id, drawer, max_size, ctx),
        }
    }
}

impl View for Stack {
    fn draw(&self, draw_ctx: DrawContext) {
        let DrawContext {
            id,
            drawer,
            size: max_size,
            ctx,
        } = draw_ctx;
        self.update_layout(id, drawer, max_size, ctx);
        if self.views.is_empty() {
            return;
        }
        self.draw_views(id, drawer, max_size, ctx);
    }

    fn process_event(&mut self, _event: &Event, _ctx: &mut Context) -> bool {
        false
    }

    fn get_min_size(&self, _drawer: &mut Piet, ctx: &mut Context) -> Size {
        Size::default()
    }

    fn is_flexible(&self) -> bool {
        true
    }
}
