use crate::*;

#[derive(Clone)]
pub struct EmptyView {}

impl View for EmptyView {
    fn draw(&self, _path: &mut IdPath, _cx: &mut Context, _vger: &mut Drawer) {}
    fn layout(
        &self,
        _path: &mut IdPath,
        sz: Rect,
        _cx: &mut Context,
        _dcx: &mut DrawerState,
    ) -> Rect {
        sz.with_size((0.0, 0.0))
    }
}

impl private::Sealed for EmptyView {}
