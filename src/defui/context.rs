use super::*;
use shell::{
    kurbo::{Affine, Point, Size, Vec2},
    KbKey, MouseButton,
};
use std::{collections::HashMap, hash::Hash};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct ViewId(pub(crate) usize);

impl Hash for ViewId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.0);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    offset: Affine,
    size: Size,
}

impl Layout {
    pub fn new(offset: Affine, size: Size) -> Self {
        Self { offset, size }
    }

    pub fn intersects(&self, point: Point) -> bool {
        let x = self.size.width;
        let y = self.size.height;
        let point = point.to_vec2() - self.offset.translation();
        let negative_point = -self.size.to_vec2() + point;
        let vectors = [
            Vec2::new(x, 0.0),
            Vec2::new(0.0, -y),
            Vec2::new(-x, 0.0),
            Vec2::new(0.0, y),
        ];
        vectors[0].cross(point) > 0.0
            && vectors[1].cross(point) > 0.0
            && vectors[2].cross(negative_point) > 0.0
            && vectors[3].cross(negative_point) > 0.0
    }
}

struct ViewState {}

pub struct Context {
    pub(crate) arena: HashMap<ViewId, Box<dyn View>>,
    pub(crate) layouts: HashMap<ViewId, Layout>,
    pub(crate) last_id: usize,
    pub(crate) pointer: Vec2,
    pub(crate) pressed_mb: HashMap<MouseButton, bool>,
    pub(crate) pressed_keys: HashMap<KbKey, bool>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            arena: HashMap::new(),
            layouts: HashMap::new(),
            last_id: 0,
            pointer: Vec2::new(0.0, 0.0),
            pressed_mb: HashMap::new(),
            pressed_keys: HashMap::new(),
        }
    }

    /// Used on view initiation
    pub fn push_view<V: View + 'static>(&mut self, view: V) -> ViewId {
        self.last_id += 1;
        self.arena.insert(ViewId(self.last_id), Box::new(view));
        ViewId(self.last_id)
    }

    /// Used to get view by id when rendering
    pub fn get_view(&mut self, id: ViewId) -> Option<Box<dyn View>> {
        self.arena.remove(&id)
    }

    /// Used to push view back to the arena after rendering
    pub fn return_view(&mut self, id: ViewId, view: Box<dyn View>) {
        self.arena.insert(id, view);
    }

    pub fn set_layout(&mut self, id: ViewId, layout: Layout) {
        if let Some(current) = self.layouts.get_mut(&id) {
            *current = layout;
        } else {
            self.layouts.insert(id, layout);
        }
    }

    pub fn get_layout(&self, id: ViewId) -> Option<&Layout> {
        self.layouts.get(&id)
    }
}
