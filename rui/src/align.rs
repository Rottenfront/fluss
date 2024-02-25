use crate::*;

pub enum HAlignment {
    Leading,
    Center,
    Trailing,
}

pub fn align_h(child: RRect, parent: RRect, align: HAlignment) -> Vec2 {
    let c_off = parent.center() - child.center();
    match align {
        HAlignment::Leading => (parent.rect().min_x() - child.rect().min_x(), c_off.y).into(),
        HAlignment::Center => c_off,
        HAlignment::Trailing => (parent.rect().max_x() - child.rect().max_x(), c_off.y).into(),
    }
}

pub enum VAlignment {
    Top,
    Middle,
    Bottom,
}

pub fn align_v(child: RRect, parent: RRect, align: VAlignment) -> Vec2 {
    let c_off = parent.center() - child.center();
    match align {
        VAlignment::Top => (c_off.x, parent.rect().max_y() - child.rect().max_y()).into(),
        VAlignment::Middle => c_off,
        VAlignment::Bottom => (c_off.x, parent.rect().min_y() - child.rect().min_y()).into(),
    }
}

pub fn align(child: RRect, parent: RRect, halign: HAlignment, valign: VAlignment) -> Vec2 {
    let c_off = parent.center() - child.center();
    Vec2::new(
        match halign {
            HAlignment::Leading => parent.rect().min_x() - child.rect().min_x(),
            HAlignment::Center => c_off.x,
            HAlignment::Trailing => parent.rect().max_x() - child.rect().max_x(),
        },
        match valign {
            VAlignment::Top => parent.rect().max_y() - child.rect().max_y(),
            VAlignment::Middle => c_off.y,
            VAlignment::Bottom => parent.rect().min_y() - child.rect().min_y(),
        },
    )
}

#[cfg(test)]
mod tests {

    use super::*;

    fn rect<A, B>(origin: A, size: B) -> RRect
    where
        A: Into<Point>,
        B: Into<Size>,
    {
        RRect::from_origin_size(origin, size, RectRadii::default())
    }

    #[test]
    fn test_align_h() {
        let parent = rect((0.0, 0.0), (10.0, 10.0));

        let off = align_h(rect((0.0, 0.0), (1.0, 1.0)), parent, HAlignment::Center);
        assert_eq!(off.x, 4.5);
        assert_eq!(off.y, 4.5);

        let off = align_h(rect((0.0, 0.0), (1.0, 1.0)), parent, HAlignment::Leading);
        assert_eq!(off.x, 0.0);
        assert_eq!(off.y, 4.5);

        let off = align_h(rect((0.0, 0.0), (1.0, 1.0)), parent, HAlignment::Trailing);
        assert_eq!(off.x, 9.0);
        assert_eq!(off.y, 4.5);
    }

    #[test]
    fn test_align_v() {
        let parent = rect((0.0, 0.0), (10.0, 10.0));

        let off = align_v(rect((0.0, 0.0), (1.0, 1.0)), parent, VAlignment::Middle);
        assert_eq!(off.x, 4.5);
        assert_eq!(off.y, 4.5);

        let off = align_v(rect((0.0, 0.0), (1.0, 1.0)), parent, VAlignment::Bottom);
        assert_eq!(off.x, 4.5);
        assert_eq!(off.y, 0.0);

        let off = align_v(rect((0.0, 0.0), (1.0, 1.0)), parent, VAlignment::Top);
        assert_eq!(off.x, 4.5);
        assert_eq!(off.y, 9.0);
    }

    #[test]
    fn test_align() {
        let parent = rect((0.0, 0.0), (10.0, 10.0));

        let off = align(
            rect((0.0, 0.0), (1.0, 1.0)),
            parent,
            HAlignment::Center,
            VAlignment::Middle,
        );
        assert_eq!(off.x, 4.5);
        assert_eq!(off.y, 4.5);

        let off = align(
            rect((0.0, 0.0), (1.0, 1.0)),
            parent,
            HAlignment::Leading,
            VAlignment::Bottom,
        );
        assert_eq!(off.x, 0.0);
        assert_eq!(off.y, 0.0);
    }
}
