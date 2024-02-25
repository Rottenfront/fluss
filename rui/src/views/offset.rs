use crate::*;
use std::any::Any;

/// Struct for the `offset` modifier.
pub struct Offset<V> {
    child: V,
    child_layout: FBinding<Rect>,
    offset: Vec2,
}

impl<V> View for Offset<V>
where
    V: View,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        path.push(0);
        self.child
            .process(&event.offset(-self.offset), path, cx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, cx: &mut Context, vger: &mut Drawer) {
        path.push(0);
        self.child.draw(path, cx, vger);
        path.pop();
    }

    fn layout(&self, path: &mut IdPath, sz: Rect, cx: &mut Context, dcx: &mut DrawerState) -> Rect {
        path.push(0);
        let sz = self.child.layout(
            path,
            Rect::new(sz.x0 + self.offset.x, sz.y0 + self.offset.y, sz.x1, sz.y1),
            cx,
            dcx,
        );
        self.child_layout.set(sz);
        path.pop();
        sz
    }

    fn dirty(&self, path: &mut IdPath, xform: Vec2, cx: &mut Context) {
        path.push(0);
        self.child.dirty(path, xform + self.offset, cx);
        path.pop();
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, cx: &mut Context) -> Option<ViewId> {
        path.push(0);
        let hit_id = self.child.hittest(path, pt - self.offset, cx);
        path.pop();
        hit_id
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

impl<V> Offset<V>
where
    V: View,
{
    pub fn new(child: V, offset: Vec2) -> Self {
        Self {
            child,
            child_layout: fbind(Rect::ZERO),
            offset,
        }
    }
}

impl<V> private::Sealed for Offset<V> {}
