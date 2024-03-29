use super::*;
use shell::{
    kurbo::{Affine, Point, Size, Vec2},
    piet::Piet,
    MouseButton,
};
use std::{collections::HashMap, hash::Hash, sync::atomic::AtomicUsize};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct ViewId(pub(crate) usize);

impl Hash for ViewId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.0);
    }
}

static mut LAST_VIEW_ID: AtomicUsize = AtomicUsize::new(0);

pub fn new_id() -> ViewId {
    unsafe {
        let last_id = LAST_VIEW_ID.get_mut();
        *last_id += 1;
        ViewId(*last_id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Layout {
    offset: Affine,
    size: Size,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub(crate) struct ViewState {
    pub(crate) layout: Layout,
    pub(crate) parent: Option<ViewId>,
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
    pub(super) view_states: HashMap<ViewId, ViewState>,
    pub(super) pointer: Vec2,
    pub(super) pressed_mb: HashMap<MouseButton, bool>,
    pub(super) actions: Vec<Action>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            view_states: HashMap::new(),
            pointer: Vec2::new(0.0, 0.0),
            pressed_mb: HashMap::new(),
            actions: Vec::new(),
        }
    }

    pub fn push_action(&mut self, action: Action) {
        self.actions.push(action)
    }

    pub fn set_layout(&mut self, id: ViewId, layout: Layout) {
        if let Some(current) = self.view_states.get_mut(&id) {
            current.layout = layout;
        } else {
            self.view_states.insert(
                id,
                ViewState {
                    layout,
                    parent: None,
                },
            );
        }
    }

    pub fn set_parent_view(&mut self, id: ViewId, parent: ViewId) {
        if let Some(current) = self.view_states.get_mut(&id) {
            current.parent = Some(parent);
        } else {
            self.view_states.insert(
                id,
                ViewState {
                    layout: Default::default(),
                    parent: Some(parent),
                },
            );
        }
    }

    pub fn get_layout(&self, id: ViewId) -> Option<Layout> {
        self.view_states.get(&id).map(|state| state.layout)
    }

    pub fn get_parent_view(&self, id: ViewId) -> Option<ViewId> {
        match self.view_states.get(&id).map(|state| &state.parent) {
            Some(parent) => *parent,
            None => None,
        }
    }
}

pub struct DrawContext<'a, 'b, 'c> {
    pub drawer: &'a mut Piet<'b>,
    pub ctx: &'c mut Context,
    pub size: Size,
}
