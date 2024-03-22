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
    direction: Binding<StackDirection>,
    views: Binding<Vec<ViewId>>,
}

impl Stack {
    pub fn vstack(views: Binding<Vec<ViewId>>) -> Self {
        Self {
            direction: bind(StackDirection::Vertical),
            views,
        }
    }

    pub fn hstack(views: Binding<Vec<ViewId>>) -> Self {
        Self {
            direction: bind(StackDirection::Horizontal),
            views,
        }
    }

    pub fn zstack(views: Binding<Vec<ViewId>>) -> Self {
        Self {
            direction: bind(StackDirection::Depth),
            views,
        }
    }

    fn update_layout(&self, id: ViewId, drawer: &mut Piet, max_size: Size, ctx: &mut Context) {
        let offset = drawer.current_transform();
        ctx.set_layout(id, Layout::new(offset, max_size));
    }

    fn draw_vertical(
        &self,
        views: Vec<ViewId>,
        drawer: &mut Piet,
        max_size: Size,
        ctx: &mut Context,
    ) {
        let height = max_size.height / (views.len() as f64);
        let mut current_offset = 0.0;
        for id in views {
            let view = match ctx.get_view(id) {
                None => continue,
                Some(view) => view,
            };
            let _ = drawer.save();
            drawer.transform(Affine::translate((0.0, current_offset)));
            view.draw(id, drawer, Size::new(max_size.width, height), ctx);
            current_offset += height;
            let _ = drawer.restore();
            ctx.return_view(id, view);
        }
    }

    fn draw_horizontal(
        &self,
        views: Vec<ViewId>,
        drawer: &mut Piet,
        max_size: Size,
        ctx: &mut Context,
    ) {
        let width = max_size.width / (views.len() as f64);
        let mut current_offset = 0.0;
        for id in views {
            let view = match ctx.get_view(id) {
                None => continue,
                Some(view) => view,
            };
            let _ = drawer.save();
            drawer.transform(Affine::translate((current_offset, 0.0)));
            view.draw(id, drawer, Size::new(width, max_size.height), ctx);
            current_offset += width;
            let _ = drawer.restore();
            ctx.return_view(id, view);
        }
    }

    fn draw_depth(&self, views: Vec<ViewId>, drawer: &mut Piet, max_size: Size, ctx: &mut Context) {
        for id in views {
            let view = match ctx.get_view(id) {
                None => continue,
                Some(view) => view,
            };
            view.draw(id, drawer, max_size, ctx);
            ctx.return_view(id, view);
        }
    }

    fn draw_views(&self, views: Vec<ViewId>, drawer: &mut Piet, max_size: Size, ctx: &mut Context) {
        match self.direction.get() {
            StackDirection::Vertical => self.draw_vertical(views, drawer, max_size, ctx),
            StackDirection::Horizontal => self.draw_horizontal(views, drawer, max_size, ctx),
            StackDirection::Depth => self.draw_depth(views, drawer, max_size, ctx),
        }
    }
}

impl View for Stack {
    fn draw(&self, id: ViewId, drawer: &mut Piet, max_size: Size, ctx: &mut Context) {
        self.update_layout(id, drawer, max_size, ctx);
        let views = self.views.get();
        if views.is_empty() {
            return;
        }
        self.draw_views(views, drawer, max_size, ctx);
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
