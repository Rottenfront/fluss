use std::collections::HashMap;

use super::*;
use shell::{
    kurbo::{Affine, Size},
    piet::{Piet, RenderContext},
    MouseButton,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackDirection {
    Vertical,
    Horizontal,
    Depth,
}

pub struct Stack {
    id: ViewId,
    direction: StackDirection,
    views: Vec<Box<dyn View>>,
    /// Saves the last target of MousePress event
    pressed_mb_target: HashMap<MouseButton, usize>,
}

impl Stack {
    pub fn new(direction: StackDirection, views: Vec<Box<dyn View>>) -> Self {
        Self {
            id: new_id(),
            direction,
            views,
            pressed_mb_target: HashMap::new(),
        }
    }
    pub fn vstack(views: Vec<Box<dyn View>>) -> Self {
        Self::new(StackDirection::Vertical, views)
    }
    pub fn hstack(views: Vec<Box<dyn View>>) -> Self {
        Self::new(StackDirection::Horizontal, views)
    }
    pub fn zstack(views: Vec<Box<dyn View>>) -> Self {
        Self::new(StackDirection::Depth, views)
    }

    fn draw_vertical(&self, drawer: &mut Piet, max_size: Size, ctx: &mut Context) {
        let height = max_size.height / (self.views.len() as f64);
        let mut current_offset = 0.0;
        for view in &self.views {
            view.update_parent(self.get_id(), ctx);
            let _ = drawer.save();
            drawer.transform(Affine::translate((0.0, current_offset)));
            view.draw(DrawContext {
                drawer,
                size: Size::new(max_size.width, height),
                ctx,
            });
            current_offset += height;
            let _ = drawer.restore();
        }
    }

    fn draw_horizontal(&self, drawer: &mut Piet, max_size: Size, ctx: &mut Context) {
        let width = max_size.width / (self.views.len() as f64);
        let mut current_offset = 0.0;
        for view in &self.views {
            view.update_parent(self.get_id(), ctx);
            let _ = drawer.save();
            drawer.transform(Affine::translate((current_offset, 0.0)));
            view.draw(DrawContext {
                drawer,
                size: Size::new(width, max_size.height),
                ctx,
            });
            current_offset += width;

            let _ = drawer.restore();
        }
    }

    fn draw_depth(&self, drawer: &mut Piet, max_size: Size, ctx: &mut Context) {
        for view in &self.views {
            view.update_parent(self.get_id(), ctx);
            view.draw(DrawContext {
                drawer,
                size: max_size,
                ctx,
            });
        }
    }

    fn draw_views(&self, drawer: &mut Piet, max_size: Size, ctx: &mut Context) {
        match self.direction {
            StackDirection::Vertical => self.draw_vertical(drawer, max_size, ctx),
            StackDirection::Horizontal => self.draw_horizontal(drawer, max_size, ctx),
            StackDirection::Depth => self.draw_depth(drawer, max_size, ctx),
        }
    }

    fn process_update(&mut self, ctx: &mut Context) -> bool {
        let mut processed = false;
        for view in &mut self.views {
            processed |= view.process_event(&Event::Update, ctx);
        }
        processed
    }

    fn process_mouse_press(&mut self, event: &Event, ctx: &mut Context) -> bool {
        let Event::MousePress { button, pos } = event else {
            return false;
        };
        if self.direction == StackDirection::Depth {
            let mut processed = false;
            for view in &mut self.views {
                processed |= view.process_event(event, ctx);
            }
            processed
        } else {
            for (id, view) in self.views.iter_mut().enumerate() {
                if let Some(true) = view.get_layout(ctx).map(|layout| layout.intersects(*pos)) {
                    self.pressed_mb_target.insert(*button, id);
                    return view.process_event(
                        &Event::MousePress {
                            button: *button,
                            pos: *pos,
                        },
                        ctx,
                    );
                }
            }
            false
        }
    }

    fn process_mouse_unpress(&mut self, event: &Event, ctx: &mut Context) -> bool {
        let Event::MouseUnpress { button, pos } = event else {
            return false;
        };
        if self.direction == StackDirection::Depth {
            let mut processed = false;
            for view in &mut self.views {
                processed |= view.process_event(event, ctx);
            }
            processed
        } else {
            let (current_target_id, processed) = 'out: {
                for (id, view) in self.views.iter_mut().enumerate() {
                    if let Some(true) = view.get_layout(ctx).map(|layout| layout.intersects(*pos)) {
                        break 'out (id as isize, view.process_event(&event, ctx));
                    }
                }
                (-1, false)
            };
            processed
                || if let Some(id) = self.pressed_mb_target.remove(button) {
                    if id as isize != current_target_id {
                        self.views[id].process_event(event, ctx)
                    } else {
                        false
                    }
                } else {
                    false
                }
        }
    }
}

impl View for Stack {
    fn draw(&self, draw_ctx: DrawContext) {
        let DrawContext {
            drawer,
            size: max_size,
            ctx,
        } = draw_ctx;
        self.update_layout(Layout::new(drawer.current_transform(), max_size), ctx);
        if self.views.is_empty() {
            return;
        }
        self.draw_views(drawer, max_size, ctx);
    }

    fn get_id(&self) -> ViewId {
        self.id
    }

    fn process_event(&mut self, event: &Event, ctx: &mut Context) -> bool {
        match event {
            Event::MousePress { .. } => self.process_mouse_press(event, ctx),
            Event::MouseUnpress { .. } => self.process_mouse_unpress(event, ctx),
            Event::Update => self.process_update(ctx),
        }
    }

    fn get_min_size(&self, _drawer: &mut Piet, ctx: &mut Context) -> Size {
        Size::default()
    }

    fn is_flexible(&self) -> bool {
        true
    }
}

#[macro_export]
macro_rules! zstack {
    // The pattern for a single `eval`
    {$($view:expr),+} => {
        {
            let mut views = vec![];
            $(
                views.push(Box::new($view) as Box<(dyn View + 'static)>);
            )+
            Stack::zstack(views)
        }
    };
}

#[macro_export]
macro_rules! hstack {
    // The pattern for a single `eval`
    {$($view:expr),+} => {
        {
            let mut views = vec![];
            $(
                views.push(Box::new($view) as Box<(dyn View + 'static)>);
            )+
            Stack::hstack(views)
        }
    };
}

#[macro_export]
macro_rules! vstack {
    // The pattern for a single `eval`
    {$($view:expr),+} => {
        {
            let mut views = vec![];
            $(
                views.push(Box::new($view) as Box<(dyn View + 'static)>);
            )+
            Stack::vstack(views)
        }
    };
}
