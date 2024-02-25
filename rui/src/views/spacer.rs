use crate::*;

#[derive(Clone)]
pub struct Spacer {}

impl View for Spacer {
    fn draw(&self, path: &mut IdPath, cx: &mut Context, vger: &mut Drawer) {}
    fn layout(&self, path: &mut IdPath, sz: Rect, cx: &mut Context, dcx: &mut DrawerState) -> Rect {
        sz.with_size((0.0, 0.0))
    }

    fn is_flexible(&self) -> bool {
        true
    }
}

impl private::Sealed for Spacer {}

/// Inserts a flexible space in a stack.
pub fn spacer() -> Spacer {
    Spacer {}
}
