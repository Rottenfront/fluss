use crate::*;
use std::any::Any;

struct EnvView<S, V, F> {
    func: F,
    phantom_s: std::marker::PhantomData<S>,
    phantom_v: std::marker::PhantomData<V>,
}

impl<S, V, F> View for EnvView<S, V, F>
where
    V: View,
    S: Clone + Default + 'static,
    F: Fn(S, &mut Context) -> V + 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        path.push(0);
        (self.func)(cx.init_env(&S::default), cx).process(event, path, cx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, cx: &mut Context, vger: &mut Drawer) {
        path.push(0);
        (self.func)(cx.init_env(&S::default), cx).draw(path, cx, vger);
        path.pop();
    }

    fn layout(&self, path: &mut IdPath, sz: Rect, cx: &mut Context, dcx: &mut DrawerState) -> Rect {
        path.push(0);
        let sz = (self.func)(cx.init_env(&S::default), cx).layout(path, sz, cx, dcx);
        path.pop();
        sz
    }

    fn dirty(&self, path: &mut IdPath, xform: Vec2, cx: &mut Context) {
        path.push(0);
        (self.func)(cx.init_env(&S::default), cx).dirty(path, xform, cx);
        path.pop();
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, cx: &mut Context) -> Option<ViewId> {
        path.push(0);
        let vid = (self.func)(cx.init_env(&S::default), cx).hittest(path, pt, cx);
        path.pop();
        vid
    }

    fn commands(&self, path: &mut IdPath, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        path.push(0);
        (self.func)(cx.init_env(&S::default), cx).commands(path, cx, cmds);
        path.pop();
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(cx.view_id(path));
        path.push(0);
        (self.func)(cx.init_env(&S::default), cx).gc(path, cx, map);
        path.pop();
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        path.push(0);
        let node_id = (self.func)(cx.init_env(&S::default), cx).access(path, cx, nodes);
        path.pop();
        node_id
    }
}

impl<S, V, F> private::Sealed for EnvView<S, V, F> {}

/// Reads from the environment.
pub fn env<S: Clone + Default + 'static, V: View, F: Fn(S, &mut Context) -> V + 'static>(
    f: F,
) -> impl View {
    EnvView {
        func: f,
        phantom_s: Default::default(),
        phantom_v: Default::default(),
    }
}

/// Struct for the `env` modifier.
pub struct SetenvView<V, E> {
    child: V,
    env_val: E,
}

impl<V, E> SetenvView<V, E>
where
    V: View,
    E: Clone + 'static,
{
    pub fn new(child: V, env_val: E) -> Self {
        Self { child, env_val }
    }
}

impl<V, E> View for SetenvView<V, E>
where
    V: View,
    E: Clone + 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let old = cx.set_env(&self.env_val);
        path.push(0);
        self.child.process(event, path, cx, actions);
        path.pop();
        old.and_then(|s| cx.set_env(&s));
    }

    fn draw(&self, path: &mut IdPath, cx: &mut Context, vger: &mut Drawer) {
        let old = cx.set_env(&self.env_val);
        path.push(0);
        self.child.draw(path, cx, vger);
        path.pop();
        old.and_then(|s| cx.set_env(&s));
    }

    fn layout(&self, path: &mut IdPath, sz: Rect, cx: &mut Context, dcx: &mut DrawerState) -> Rect {
        let old = cx.set_env(&self.env_val);
        path.push(0);
        let sz = self.child.layout(path, sz, cx, dcx);
        path.pop();
        old.and_then(|s| cx.set_env(&s));
        sz
    }

    fn dirty(&self, path: &mut IdPath, xform: Vec2, cx: &mut Context) {
        let old = cx.set_env(&self.env_val);
        path.push(0);
        self.child.dirty(path, xform, cx);
        path.pop();
        old.and_then(|s| cx.set_env(&s));
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, cx: &mut Context) -> Option<ViewId> {
        let old = cx.set_env(&self.env_val);
        path.push(0);
        let r = self.child.hittest(path, pt, cx);
        path.pop();
        old.and_then(|s| cx.set_env(&s));
        r
    }

    fn commands(&self, path: &mut IdPath, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let old = cx.set_env(&self.env_val);
        path.push(0);
        self.child.commands(path, cx, cmds);
        path.pop();
        old.and_then(|s| cx.set_env(&s));
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        let old = cx.set_env(&self.env_val);
        path.push(0);
        self.child.gc(path, cx, map);
        path.pop();
        old.and_then(|s| cx.set_env(&s));
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let old = cx.set_env(&self.env_val);
        path.push(0);
        let r = self.child.access(path, cx, nodes);
        path.pop();
        old.and_then(|s| cx.set_env(&s));
        r
    }
}

impl<V, E> private::Sealed for SetenvView<V, E> {}
