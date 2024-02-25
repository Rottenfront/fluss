use crate::*;

/// Struct for `canvas`
#[derive(Clone)]
pub struct Canvas<F> {
    func: F,
}

impl<F> View for Canvas<F>
where
    F: Fn(&mut Context, Rect, &mut Drawer) + 'static,
{
    fn draw(&self, path: &mut IdPath, cx: &mut Context, vger: &mut Drawer) {
        let rect = cx.get_layout(path);

        vger.save();
        (self.func)(cx, rect, vger);
        vger.restore();
    }

    fn layout(&self, path: &mut IdPath, sz: Rect, cx: &mut Context, dcx: &mut DrawerState) -> Rect {
        cx.update_layout(path, sz);
        sz
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, cx: &mut Context) -> Option<ViewId> {
        let rect = cx.get_layout(path);

        if rect.contains(pt) {
            Some(cx.view_id(path))
        } else {
            None
        }
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(cx.view_id(path));
    }
}

/// Canvas for GPU drawing with Vger. See https://github.com/audulus/vger-rs.
pub fn canvas<F: Fn(&mut Context, Rect, &mut Drawer) + 'static>(f: F) -> impl View {
    Canvas { func: f }
}

impl<F> private::Sealed for Canvas<F> {}
