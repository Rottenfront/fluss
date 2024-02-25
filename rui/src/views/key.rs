use crate::*;
use std::any::Any;

/// Struct for the `key` modifier.
pub struct KeyView<V, F> {
    child: V,
    func: F,
}

impl<V, F, A> KeyView<V, F>
where
    V: View,
    F: Fn(&mut Context, Key) -> A + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        KeyView { child: v, func: f }
    }
}

impl<V, F, A> View for KeyView<V, F>
where
    V: View,
    F: Fn(&mut Context, Key) -> A + 'static,
    A: 'static,
{
    fn process(
        &self,
        event: &Event,
        _path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        if let Event::Key(key) = &event {
            actions.push(Box::new((self.func)(cx, key.clone())));
        }
    }

    fn draw(&self, path: &mut IdPath, cx: &mut Context, vger: &mut Drawer) {
        path.push(0);
        self.child.draw(path, cx, vger);
        path.pop();
    }

    fn layout(&self, path: &mut IdPath, sz: Rect, cx: &mut Context, dcx: &mut DrawerState) -> Rect {
        path.push(0);
        let sz = self.child.layout(path, sz, cx, dcx);
        path.pop();
        sz
    }

    fn dirty(&self, path: &mut IdPath, xform: Vec2, cx: &mut Context) {
        path.push(0);
        self.child.dirty(path, xform, cx);
        path.pop();
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, cx: &mut Context) -> Option<ViewId> {
        path.push(0);
        let id = self.child.hittest(path, pt, cx);
        path.pop();
        id
    }

    fn commands(&self, path: &mut IdPath, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        path.push(0);
        self.child.commands(path, cx, cmds);
        path.pop();
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        path.push(0);
        self.child.gc(path, cx, map);
        path.pop();
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        path.push(0);
        let node_id = self.child.access(path, cx, nodes);
        path.pop();
        node_id
    }
}

impl<V, F> private::Sealed for KeyView<V, F> {}
