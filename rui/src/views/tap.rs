use crate::*;
use std::any::Any;

pub trait TapFn {
    fn call(
        &self,
        cx: &mut Context,
        pt: Point,
        button: Option<MouseButton>,
        actions: &mut Vec<Box<dyn Any>>,
    );
}

pub struct TapFunc<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context, Point, Option<MouseButton>) -> A> TapFn for TapFunc<F> {
    fn call(
        &self,
        cx: &mut Context,
        pt: Point,
        button: Option<MouseButton>,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        actions.push(Box::new((self.f)(cx, pt, button)))
    }
}

pub struct TapAdapter<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context) -> A> TapFn for TapAdapter<F> {
    fn call(
        &self,
        cx: &mut Context,
        _pt: Point,
        _button: Option<MouseButton>,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        actions.push(Box::new((self.f)(cx)))
    }
}

pub struct TapActionAdapter<A> {
    pub action: A,
}

impl<A: Clone + 'static> TapFn for TapActionAdapter<A> {
    fn call(
        &self,
        _cx: &mut Context,
        _pt: Point,
        _button: Option<MouseButton>,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        actions.push(Box::new(self.action.clone()))
    }
}

/// Struct for the `tap` gesture.
pub struct Tap<V: View, F> {
    /// Child view tree.
    child: V,

    /// Called when a tap occurs.
    func: F,
}

impl<V, F> Tap<V, F>
where
    V: View,
    F: TapFn + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self { child: v, func: f }
    }
}

impl<V, F> View for Tap<V, F>
where
    V: View,
    F: TapFn + 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let vid = cx.view_id(path);
        match &event {
            Event::TouchBegin { id, position } => {
                if self.hittest(path, *position, cx).is_some() {
                    cx.touches[*id] = vid;
                }
            }
            Event::TouchEnd { id, position } => {
                if cx.touches[*id] == vid {
                    cx.touches[*id] = ViewId::default();
                    self.func.call(cx, *position, cx.mouse_button, actions)
                }
            }
            _ => (),
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

impl<V, F> private::Sealed for Tap<V, F> where V: View {}
