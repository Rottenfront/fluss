use super::*;
use shell::{
    kurbo::{Affine, Point, Size, Vec2},
    piet::Piet,
    MouseButton,
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

pub struct Context {
    pub(super) arena: HashMap<ViewId, Box<dyn View>>,
    pub(super) layouts: HashMap<ViewId, Layout>,
    pub(super) last_id: usize,
    pub(super) pointer: Vec2,
    pub(super) pressed_mb: HashMap<MouseButton, (bool, Vec<ViewId>)>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            arena: HashMap::new(),
            layouts: HashMap::new(),
            last_id: 0,
            pointer: Vec2::new(0.0, 0.0),
            pressed_mb: HashMap::new(),
        }
    }

    pub(crate) fn set_root_view<V: View + 'static>(&mut self, view: V) -> ViewId {
        self.arena.insert(ViewId(0), Box::new(view));
        ViewId(0)
    }

    /// Used on view initiation
    pub fn push_view<V: View + 'static>(&mut self, view: V) -> ViewId {
        self.last_id += 1;
        self.arena.insert(ViewId(self.last_id), Box::new(view));
        ViewId(self.last_id)
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

    pub fn map_view<T: Default, F: FnMut(&mut Box<dyn View>, &mut Self) -> T>(
        &mut self,
        id: ViewId,
        f: &mut F,
    ) -> T {
        let mut view = match self.arena.remove(&id) {
            None => return Default::default(),
            Some(view) => view,
        };
        let res = f(&mut view, self);
        self.arena.insert(id, view);
        res
    }
}

pub struct DrawContext<'a, 'b, 'c> {
    pub drawer: &'a mut Piet<'b>,
    pub ctx: &'c mut Context,
    pub size: Size,
    pub id: ViewId,
}
