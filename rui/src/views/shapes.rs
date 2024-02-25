use crate::*;

/// Struct for `circle`.
#[derive(Clone)]
pub struct CircleView {
    paint: Paint,
}

impl CircleView {
    fn geom(&self, path: &IdPath, cx: &mut Context) -> (Point, f64) {
        let rect = cx.get_layout(path);

        (rect.center(), rect.width().min(rect.height()) / 2.0)
    }
    /// Sets the fill color for the rectangle.
    pub fn color(self, color: Color) -> CircleView {
        CircleView {
            paint: Paint::Color(color),
        }
    }
}

impl View for CircleView {
    fn draw(&self, path: &mut IdPath, cx: &mut Context, vger: &mut Drawer) {
        let (center, radius) = self.geom(path, cx);
        let paint = vger.state().create_fast_paint(self.paint.clone()).unwrap();

        vger.draw_circle(center, radius, paint);
    }

    fn layout(&self, path: &mut IdPath, sz: Rect, cx: &mut Context, dcx: &mut DrawerState) -> Rect {
        cx.update_layout(path, sz);
        sz
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, cx: &mut Context) -> Option<ViewId> {
        let (center, radius) = self.geom(path, cx);

        if pt.distance(center) < radius {
            Some(cx.view_id(path))
        } else {
            None
        }
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(cx.view_id(path));
    }
}

impl private::Sealed for CircleView {}

/// Renders a circle which expands to fill available space.
pub fn circle() -> CircleView {
    CircleView {
        paint: Paint::Color(Color::CYAN),
    }
}

/// Struct for `rectangle`.
#[derive(Clone)]
pub struct RectView {
    corner_radius: f64,
    paint: Paint,
}

impl RectView {
    fn geom(&self, path: &IdPath, cx: &mut Context) -> RRect {
        RRect::from_rect(
            cx.get_layout(path),
            RectRadii::from_single_radius(self.corner_radius),
        )
    }

    /// Sets the fill color for the rectangle.
    pub fn color(self, color: Color) -> RectView {
        RectView {
            corner_radius: self.corner_radius,
            paint: Paint::Color(color),
        }
    }

    /// Sets the rectangle's corner radius.
    pub fn corner_radius(self, radius: f64) -> RectView {
        RectView {
            corner_radius: radius,
            paint: self.paint,
        }
    }
}

impl View for RectView {
    fn draw(&self, path: &mut IdPath, cx: &mut Context, vger: &mut Drawer) {
        let rect = self.geom(path, cx);
        let paint = vger.state().create_fast_paint(self.paint.clone()).unwrap();
        vger.draw_rect(&rect, paint);
    }

    fn layout(&self, path: &mut IdPath, sz: Rect, cx: &mut Context, dcx: &mut DrawerState) -> Rect {
        cx.update_layout(path, sz);
        sz
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, cx: &mut Context) -> Option<ViewId> {
        let rect = self.geom(path, cx);

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

impl private::Sealed for RectView {}

/// Renders a rectangle which expands to fill available space.
pub fn rectangle() -> RectView {
    RectView {
        corner_radius: 0.0,
        paint: Paint::Color(Color::CYAN),
    }
}
