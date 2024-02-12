use crate::*;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    ops,
};

pub const DEBUG_LAYOUT: bool = false;

pub struct ViewState {
    /// Describes nesting level of widget. Root widget has level 0
    level: usize,
    item: Box<dyn View>,
    layout: RoundedRect,
}

pub(crate) struct StateHolder {
    pub state: Box<dyn Any>,
    pub dirty: bool,
}

pub type Arena = HashMap<ViewId, ViewState>;

pub struct WindowProperties {
    pub(crate) transparent: bool,

    pub(crate) scale: f64,

    pub(crate) window_size: Size,

    /// The current title of the window
    pub(crate) window_title: String,

    /// Are we fullscreen?
    pub(crate) fullscreen: bool,
}

impl WindowProperties {
    pub fn transparency(&self) -> bool {
        self.transparent
    }

    pub fn window_size(&self) -> Size {
        self.window_size
    }

    pub fn window_title(&self) -> &str {
        self.window_title.as_ref()
    }

    pub fn fullscreen(&self) -> bool {
        self.fullscreen
    }
}

impl Default for WindowProperties {
    fn default() -> Self {
        Self {
            transparent: false,
            scale: 1.0,
            window_size: (800.0, 800.0).into(),
            window_title: "Application".to_owned(),
            fullscreen: false,
        }
    }
}

/// The Context stores all UI state. A user of the library
/// shouldn't have to interact with it directly.
pub struct Context {
    /// View information
    pub arena: Arena,

    /// Keyboard modifiers state
    pub key_mods: KeyboardModifiers,

    pub window_properties: WindowProperties,

    pub(crate) key_press_status: HashMap<Key, bool>,

    /// The view that has the keyboard focus.
    pub(crate) keyboard_focused_id: Option<ViewId>,

    pub(crate) widgets_under_cursor: Vec<ViewId>,

    /// Lock the cursor in position. Useful for dragging knobs.
    pub(crate) grab_cursor: bool,

    /// Value of grab_cursor before processing event.
    pub(crate) prev_grab_cursor: bool,

    state_map: HashMap<ViewId, StateHolder>,

    env: HashMap<TypeId, Box<dyn Any>>,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
            key_mods: KeyboardModifiers::new(),
            window_properties: Default::default(),
            key_press_status: HashMap::new(),
            keyboard_focused_id: None,
            widgets_under_cursor: vec![],
            grab_cursor: false,
            prev_grab_cursor: false,
            state_map: HashMap::new(),
            env: HashMap::new(),
        }
    }

    /// Redraw the UI
    pub fn render(&mut self, view: &impl View, drawer: &mut Drawer) {
        view.draw(
            drawer,
            self,
            Rect::from_origin_size((0.0, 0.0), self.window_properties.window_size()),
        );
    }

    /// Process a UI event.
    pub fn handle_event(
        &mut self,
        view: &mut impl View,
        drawer_state: &mut DrawerState,
        event: &Event,
    ) {
        let mut actions = vec![];
        view.handle_event(drawer_state, self, event, &mut actions);
        // TODO handle actions
    }

    pub(crate) fn set_state<S: 'static>(&mut self, id: ViewId, value: S) {
        self.state_map.insert(
            id,
            StateHolder {
                state: Box::new(value),
                dirty: false,
            },
        );
    }

    pub(crate) fn init_state<S: 'static, D: Fn() -> S + 'static>(&mut self, id: ViewId, func: &D) {
        self.state_map.entry(id).or_insert_with(|| StateHolder {
            state: Box::new((func)()),
            dirty: false,
        });
    }

    pub(crate) fn init_env<S: Clone + 'static, D: Fn() -> S + 'static>(&mut self, func: &D) -> S {
        self.env
            .entry(TypeId::of::<S>())
            .or_insert_with(|| Box::new((func)()))
            .downcast_ref::<S>()
            .unwrap()
            .clone()
    }

    pub(crate) fn set_env<S: Clone + 'static>(&mut self, value: &S) -> Option<S> {
        let typeid = TypeId::of::<S>();
        let old_value = self
            .env
            .get(&typeid)
            .map(|b| b.downcast_ref::<S>().unwrap().clone());
        self.env.insert(typeid, Box::new(value.clone()));
        old_value
    }

    pub fn get<S: 'static>(&self, id: StateHandle<S>) -> &S {
        self.state_map[&id.id].state.downcast_ref::<S>().unwrap()
    }

    pub fn get_mut<S: 'static>(&mut self, id: StateHandle<S>) -> &mut S {
        let holder = self.state_map.get_mut(&id.id).unwrap();
        holder.dirty = true;
        holder.state.downcast_mut::<S>().unwrap()
    }
}

impl<S> ops::Index<StateHandle<S>> for Context
where
    S: 'static,
{
    type Output = S;

    fn index(&self, index: StateHandle<S>) -> &S {
        self.get(index)
    }
}

impl<S> ops::IndexMut<StateHandle<S>> for Context
where
    S: 'static,
{
    fn index_mut(&mut self, index: StateHandle<S>) -> &mut S {
        self.get_mut(index)
    }
}

/// Trait for the unit of UI composition.
pub trait View: private::Sealed + 'static {
    /// Builds an AccessKit tree. The node ID for the subtree is returned. All generated nodes are accumulated.
    fn access(
        &self,
        _ctx: &mut Context,
        _nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        None
    }

    /// Draws the view
    fn draw(&self, drawer: &mut Drawer, context: &mut Context, current_box: Rect);

    /// For detecting flexible-sized things in stacks.
    fn is_flexible(&self) -> bool {
        false
    }

    fn is_scrollable(&self) -> bool {
        false
    }

    fn min_size(&self) -> Size;

    fn max_size(&self) -> Size;

    /// Processes an event.
    fn handle_event(
        &mut self,
        draw_state: &mut DrawerState,
        ctx: &mut Context,
        event: &Event,
        actions: &mut Vec<ApplicationAction>,
    );

    /// Returns the type ID of the underlying view.
    fn tid(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

#[derive(Clone)]
pub enum ApplicationAction {
    ChangeTitle(String),
    ChangeFullscreen(bool),
}
