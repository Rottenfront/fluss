use crate::*;

/// Reads or writes a value owned by a source-of-truth.
pub trait Binding<S>: Clone + Copy + 'static {
    fn get<'a>(&self, cx: &'a Context) -> &'a S;
    fn get_mut<'a>(&self, cx: &'a mut Context) -> &'a mut S;

    fn with<T>(&self, cx: &Context, f: impl FnOnce(&S) -> T) -> T {
        f(self.get(cx))
    }

    fn with_mut<T>(&self, cx: &mut Context, f: impl FnOnce(&mut S) -> T) -> T {
        f(self.get_mut(cx))
    }
}

pub fn setter<S>(binding: impl Binding<S>) -> impl Fn(S, &mut Context) {
    move |s, cx| binding.with_mut(cx, |v| *v = s)
}

pub struct Map<B, L, S, T> {
    binding: B,
    lens: L,
    phantom_s: std::marker::PhantomData<S>,
    phantom_t: std::marker::PhantomData<T>,
}

impl<B, L, S, T> Clone for Map<B, L, S, T>
where
    B: Copy,
    L: Copy,
    S: 'static,
    T: 'static,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<B, L, S, T> Copy for Map<B, L, S, T>
where
    B: Copy,
    L: Copy,
    S: 'static,
    T: 'static,
{
}

impl<S, B, L, T> Map<B, L, S, T>
where
    B: Binding<T>,
    L: Lens<T, S>,
    S: 'static,
    T: 'static,
{
    pub fn new(binding: B, lens: L) -> Self {
        Self {
            binding,
            lens,
            phantom_s: Default::default(),
            phantom_t: Default::default(),
        }
    }
}

pub fn bind<S, T>(binding: impl Binding<S>, lens: impl Lens<S, T>) -> impl Binding<T>
where
    S: 'static,
    T: 'static,
{
    Map::new(binding, lens)
}

impl<S, B, L, T> Binding<S> for Map<B, L, S, T>
where
    B: Binding<T>,
    L: Lens<T, S>,
    S: 'static,
    T: 'static,
{
    fn get<'a>(&self, cx: &'a Context) -> &'a S {
        self.lens.focus(self.binding.get(cx))
    }
    fn get_mut<'a>(&self, cx: &'a mut Context) -> &'a mut S {
        self.lens.focus_mut(self.binding.get_mut(cx))
    }
}

/// Weak reference to app state.
///
/// To get the underlying value, you'll need a `Context`, which is passed
/// to all event handlers, and functions passed to `state`.
pub struct StateHandle<S> {
    pub(crate) id: ViewId,
    phantom: std::marker::PhantomData<S>,
}

impl<S> Copy for StateHandle<S> {}

impl<S> Clone for StateHandle<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: 'static> StateHandle<S> {
    pub fn new(id: ViewId) -> Self {
        Self {
            id,
            phantom: Default::default(),
        }
    }

    /// Makes it convenient to get a function to set the value.
    pub fn setter(self) -> impl Fn(S, &mut Context) {
        move |s, cx| cx[self] = s
    }
}

impl<S: 'static> Binding<S> for StateHandle<S> {
    fn get<'a>(&self, cx: &'a Context) -> &'a S {
        cx.get(*self)
    }
    fn get_mut<'a>(&self, cx: &'a mut Context) -> &'a mut S {
        cx.get_mut(*self)
    }
}

#[derive(Clone)]
pub struct StateView<D, F> {
    default: D,
    func: F,
}

impl<S, V, D, F> View for StateView<D, F>
where
    V: View,
    S: 'static,
    D: Fn() -> S + 'static,
    F: Fn(StateHandle<S>, &Context) -> V + 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let id = cx.view_id(path);
        cx.init_state(id, &self.default);
        path.push(0);
        (self.func)(StateHandle::new(id), cx).process(event, path, cx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        let id = args.cx.view_id(path);
        args.cx.init_state(id, &self.default);
        path.push(0);
        (self.func)(StateHandle::new(id), args.cx).draw(path, args);
        path.pop();
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        let id = args.cx.view_id(path);
        args.cx.init_state(id, &self.default);

        // Do we need to recompute layout?
        let mut compute_layout = true;

        if let Some(deps) = args.cx.deps.get(&id) {
            let mut any_dirty = false;
            for dep in deps {
                if let Some(holder) = args.cx.state_map.get_mut(dep) {
                    if holder.dirty {
                        any_dirty = true;
                        break;
                    }
                }
            }

            compute_layout = any_dirty;
        }

        if compute_layout {
            args.cx.id_stack.push(id);

            let view = (self.func)(StateHandle::new(id), args.cx);

            path.push(0);
            let child_size = view.layout(path, args);

            // Compute layout dependencies.
            let mut deps = vec![];
            deps.append(&mut args.cx.id_stack.clone());
            view.gc(path, args.cx, &mut deps);

            path.pop();

            args.cx.deps.insert(id, deps);

            let layout_box = LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), child_size),
                offset: LocalOffset::zero(),
            };
            args.cx.update_layout(path, layout_box);

            args.cx.id_stack.pop();
        }

        args.cx.get_layout(path).rect.size
    }

    fn dirty(&self, path: &mut IdPath, xform: LocalToWorld, cx: &mut Context) {
        let default = &self.default;
        let id = cx.view_id(path);
        let holder = cx.state_map.entry(id).or_insert_with(|| StateHolder {
            state: Box::new((default)()),
            dirty: false,
        });

        if holder.dirty {
            // Add a region.
            let rect = cx.get_layout(path).rect;
            let pts: [LocalPoint; 4] = [
                rect.min(),
                [rect.max_x(), rect.min_y()].into(),
                [rect.min_x(), rect.max_y()].into(),
                rect.max(),
            ];
            let world_pts = pts.map(|p| xform.transform_point(p));
            cx.dirty_region.add_rect(WorldRect::from_points(world_pts));
        } else {
            path.push(0);
            (self.func)(StateHandle::new(id), cx).dirty(path, xform, cx);
            path.pop();
        }
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let id = cx.view_id(path);
        cx.init_state(id, &self.default);
        path.push(0);
        let hit_id = (self.func)(StateHandle::new(id), cx).hittest(path, pt, cx);
        path.pop();
        hit_id
    }

    fn commands(&self, path: &mut IdPath, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let id = cx.view_id(path);
        cx.init_state(id, &self.default);
        path.push(0);
        (self.func)(StateHandle::new(id), cx).commands(path, cx, cmds);
        path.pop();
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        let id = cx.view_id(path);
        cx.init_state(id, &self.default);
        map.push(id);
        path.push(0);
        (self.func)(StateHandle::new(id), cx).gc(path, cx, map);
        path.pop();
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let id = cx.view_id(path);
        cx.init_state(id, &self.default);
        path.push(0);
        let node_id = (self.func)(StateHandle::new(id), cx).access(path, cx, nodes);
        path.pop();
        node_id
    }
}

impl<S, F> private::Sealed for StateView<S, F> {}

/// State allows you to associate some state with a view.
/// This is what you'll use for a data model, as well as per-view state.
/// Your state should be efficiently clonable. Use Rc as necessary.
///
/// `initial` is the initial value for your state.
///
/// `f` callback which is passed a `State<S>`
pub fn state<
    S: 'static,
    V: View,
    D: Fn() -> S + 'static,
    F: Fn(StateHandle<S>, &Context) -> V + 'static,
>(
    initial: D,
    f: F,
) -> StateView<D, F> {
    StateView {
        default: initial,
        func: f,
    }
}

/// Convenience to get the context.
pub fn with_cx<V: View, F: Fn(&Context) -> V + 'static>(
    f: F,
) -> StateView<impl Fn(), impl Fn(StateHandle<()>, &Context) -> V> {
    state(|| (), move |_, cx| f(cx))
}

/// Convenience to retrieve a reference to a value in the context.
pub fn with_ref<V: View, F: Fn(&T) -> V + 'static, T>(
    binding: impl Binding<T>,
    f: F,
) -> StateView<impl Fn(), impl Fn(StateHandle<()>, &Context) -> V> {
    with_cx(move |cx| f(binding.get(cx)))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(Clone, Default)]
    struct MyState {
        x: i32,
    }

    make_lens!(MyLens, MyState, i32, x);

    #[test]
    fn test_lens() {
        let mut s = MyState { x: 0 };
        *MyLens {}.focus_mut(&mut s) = 42;
        assert_eq!(*MyLens {}.focus(&s), 42);
    }

    #[test]
    fn test_bind() {
        let mut cx = Context::new();
        let id = ViewId::default();
        cx.init_state(id, &MyState::default);
        let s = StateHandle::new(id);

        let b = bind(s, MyLens {});

        *b.get_mut(&mut cx) = 42;

        assert_eq!(*b.get(&cx), 42);
    }
}
