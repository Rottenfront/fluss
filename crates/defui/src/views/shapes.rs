use crate::*;

/// Struct for `circle`.
#[derive(Clone)]
pub struct CircleShape {
    paint: Paint,
}

impl CircleShape {
    fn geom(&self, path: &IdPath, cx: &mut Context) -> (LocalPoint, f64) {
        let rect = cx.get_layout(path).rect;

        (rect.center(), rect.size.width.min(rect.size.height) / 2.0)
    }

    pub fn color(self, color: Color) -> CircleShape {
        CircleShape {
            paint: Paint::Color(color),
        }
    }
}

impl View for CircleShape {
    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        let (center, radius) = self.geom(path, args.cx);

        let vger = &mut args.vger;
        let paint = vger.state().create_fast_paint(self.paint).unwrap();
        vger.draw_shape(
            &kurbo::Circle::new((center.x as f64, center.y as f64), radius),
            paint,
        );
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        args.cx.update_layout(
            path,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), args.sz),
                offset: LocalOffset::zero(),
            },
        );
        args.sz
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let (center, radius) = self.geom(path, cx);

        if pt.distance_to(center) < radius {
            Some(cx.view_id(path))
        } else {
            None
        }
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(cx.view_id(path));
    }
}

impl private::Sealed for CircleShape {}

/// Renders a circle which expands to fill available space.
pub fn circle() -> CircleShape {
    CircleShape {
        paint: Paint::Color(Color::CYAN),
    }
}

/// Struct for `rectangle`.
#[derive(Clone)]
pub struct RectShape {
    corner_radius: f64,
    paint: Paint,
}

impl RectShape {
    fn geom(&self, path: &IdPath, cx: &mut Context) -> LocalRect {
        cx.get_layout(path).rect
    }

    /// Sets the fill color for the rectangle.
    pub fn color(self, color: Color) -> RectShape {
        RectShape {
            corner_radius: self.corner_radius,
            paint: Paint::Color(color),
        }
    }

    /// Sets the rectangle's corner radius.
    pub fn corner_radius(self, radius: f64) -> RectShape {
        RectShape {
            corner_radius: radius,
            paint: self.paint,
        }
    }
}

impl View for RectShape {
    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        let rect = self.geom(path, args.cx);

        let vger = &mut args.vger;
        let paint = vger.state().create_fast_paint(self.paint).unwrap();
        vger.draw_shape(
            &kurbo::Rect::new(rect.min_x(), rect.min_y(), rect.max_x(), rect.max_y())
                .to_rounded_rect(kurbo::RoundedRectRadii::from_single_radius(
                    self.corner_radius,
                )),
            paint,
        );
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        args.cx.update_layout(
            path,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), args.sz),
                offset: LocalOffset::zero(),
            },
        );
        args.sz
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
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

impl private::Sealed for RectShape {}

/// Renders a rectangle which expands to fill available space.
pub fn rectangle() -> RectShape {
    RectShape {
        corner_radius: 0.0,
        paint: Paint::Color(Color::CYAN),
    }
}
