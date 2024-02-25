use crate::*;
use std::any::Any;

/// Struct for the `offset` modifier.
pub struct Padding<V> {
    child: V,
    padding: (f64, f64, f64, f64),
}

impl<V> View for Padding<V>
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
        self.child.process(
            &event.offset(-Vec2::new(self.padding.0, self.padding.1)),
            path,
            cx,
            actions,
        );
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
            Rect::new(
                sz.x0 + self.padding.0,
                sz.y0 + self.padding.1,
                sz.x1 - self.padding.2,
                sz.y1 - self.padding.3,
            ),
            cx,
            dcx,
        );
        path.pop();
        sz
    }

    fn dirty(&self, path: &mut IdPath, xform: Vec2, cx: &mut Context) {
        path.push(0);
        self.child
            .dirty(path, xform + Vec2::new(self.padding.0, self.padding.1), cx);
        path.pop();
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, cx: &mut Context) -> Option<ViewId> {
        path.push(0);
        let hit_id = self
            .child
            .hittest(path, pt - Vec2::new(self.padding.0, self.padding.1), cx);
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

pub enum PaddingParam {
    Auto,
    Px(f64),
    Spacific(f64, f64, f64, f64),
}
pub struct Auto;
impl From<Auto> for PaddingParam {
    fn from(_val: Auto) -> Self {
        PaddingParam::Auto
    }
}
impl From<f64> for PaddingParam {
    fn from(val: f64) -> Self {
        PaddingParam::Px(val)
    }
}

impl<V> Padding<V>
where
    V: View,
{
    pub fn new(child: V, param: PaddingParam) -> Self {
        Self {
            child,
            padding: match param {
                PaddingParam::Auto => (5.0, 5.0, 5.0, 5.0),
                PaddingParam::Px(px) => (px, px, px, px),
                PaddingParam::Spacific(px1, px2, px3, px4) => (px1, px2, px3, px4),
            },
        }
    }
}

impl<V> private::Sealed for Padding<V> {}
