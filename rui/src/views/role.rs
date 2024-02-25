use crate::*;
use accesskit::Role;
use std::any::Any;

/// Struct for the `role` modifier.
pub struct RoleView<V> {
    child: V,
    role: Role,
}

impl<V> RoleView<V>
where
    V: View,
{
    pub fn new(v: V, role: Role) -> Self {
        Self { child: v, role }
    }
}

impl<V> View for RoleView<V>
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
        self.child.process(event, path, cx, actions);
        path.pop();
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
        let child_aid = self.child.access(path, cx, nodes);
        path.pop();
        let aid = cx.view_id(path).access_id();
        let mut builder = accesskit::NodeBuilder::new(self.role);
        builder.set_children(match child_aid {
            Some(cid) => vec![cid],
            None => vec![],
        });
        nodes.push((aid, builder.build(&mut cx.access_node_classes)));
        Some(aid)
    }
}

impl<V> private::Sealed for RoleView<V> {}
