use crate::*;
use std::any::Any;

pub struct Clip<V> {
    child: V,
}

impl<V> Clip<V>
where
    V: View,
{
    fn geom(&self, path: &IdPath, cx: &mut Context) -> Rect {
        cx.get_layout(path)
    }

    pub fn new(child: V) -> Self {
        Self { child }
    }
}

impl<V> View for Clip<V>
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
        let rect = self.geom(path, cx);

        vger.save();
        vger.clip_rect(&RRect::from_rect(rect, RectRadii::default()));
        path.push(0);
        self.child.draw(path, cx, vger);
        path.pop();
        vger.restore();
    }

    fn layout(&self, path: &mut IdPath, sz: Rect, cx: &mut Context, dcx: &mut DrawerState) -> Rect {
        path.push(0);
        self.child.layout(path, sz, cx, dcx);
        path.pop();
        cx.update_layout(path, sz);
        // XXX: should this expand to the available space?
        sz
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, cx: &mut Context) -> Option<ViewId> {
        let rect = self.geom(path, cx);

        if rect.contains(pt) {
            // Test against children.
            path.push(0);
            let vid = self.child.hittest(path, pt, cx);
            path.pop();
            vid
        } else {
            None
        }
    }

    fn commands(&self, path: &mut IdPath, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        path.push(0);
        self.child.commands(path, cx, cmds);
        path.pop();
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(cx.view_id(path));
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

impl<V> private::Sealed for Clip<V> {}
