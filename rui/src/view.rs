use crate::*;
use std::any::{Any, TypeId};

/// Trait for the unit of UI composition.
pub trait View: private::Sealed + 'static {
    /// Builds an AccessKit tree. The node ID for the subtree is returned. All generated nodes are accumulated.
    fn access(
        &self,
        _path: &mut IdPath,
        _cx: &mut Context,
        _nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        None
    }

    /// Accumulates information about menu bar commands.
    fn commands(&self, _path: &mut IdPath, _cx: &mut Context, _cmds: &mut Vec<CommandInfo>) {}

    /// Determines dirty regions which need repainting.
    fn dirty(&self, _path: &mut IdPath, _xform: Vec2, _cx: &mut Context) {}

    /// Draws the view using vger.
    fn draw(&self, path: &mut IdPath, cx: &mut Context, vger: &mut Drawer);

    /// Gets IDs for views currently in use.
    ///
    /// Push onto map if the view stores layout or state info.
    fn gc(&self, _path: &mut IdPath, _cx: &mut Context, _map: &mut Vec<ViewId>) {}

    /// Returns the topmost view which the point intersects.
    fn hittest(&self, _path: &mut IdPath, _pt: Point, _cx: &mut Context) -> Option<ViewId> {
        None
    }

    /// For detecting flexible sized things in stacks.
    fn is_flexible(&self) -> bool {
        false
    }

    /// Lays out subviews and return the size of the view.
    ///
    /// `sz` is the available size for the view
    /// `vger` can be used to get text sizing
    ///
    /// Note that we should probably have a separate text
    /// sizing interface so we don't need a GPU and graphics
    /// context set up to test layout.
    fn layout(&self, path: &mut IdPath, sz: Rect, cx: &mut Context, dcx: &mut DrawerState) -> Rect;

    /// Processes an event.
    fn process(
        &self,
        _event: &Event,
        _path: &mut IdPath,
        _cx: &mut Context,
        _actions: &mut Vec<Box<dyn Any>>,
    ) {
    }

    /// Returns the type ID of the underlying view.
    fn tid(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}