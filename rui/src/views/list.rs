use crate::*;
use std::any::Any;
use std::hash::Hash;

pub enum ListOrientation {
    Horizontal,
    Vertical,
    Z,
}

pub struct List<ID, F> {
    orientation: ListOrientation,
    ids: Vec<ID>,
    func: F,
}

impl<ID, V, F> View for List<ID, F>
where
    ID: Hash + 'static,
    V: View,
    F: Fn(&ID) -> V + 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        for child in self.ids.iter().rev() {
            path.push(hh(child));
            let offset = cx.get_layout(path).origin().to_vec2();
            ((self.func)(child)).process(&event.offset(-offset), path, cx, actions);
            path.pop();
        }
    }

    fn draw(&self, path: &mut IdPath, cx: &mut Context, vger: &mut Drawer) {
        for child in &self.ids {
            path.push(hh(child));
            let rect = cx.get_layout(path);

            vger.clip_rect(&rect.to_rounded_rect(RectRadii::default()));

            ((self.func)(child)).draw(path, cx, vger);

            vger.restore();
            path.pop();
        }
    }

    fn layout(&self, path: &mut IdPath, sz: Rect, cx: &mut Context, dcx: &mut DrawerState) -> Rect {
        match self.orientation {
            ListOrientation::Horizontal => {
                let n = self.ids.len() as f64;
                let proposed_child_size = Size::new(sz.width() / n, sz.height());
                let offset = sz.origin();

                let mut sizes = Vec::<Size>::new();
                sizes.reserve(self.ids.len());

                let mut width_sum = 0.0;
                for child in &self.ids {
                    path.push(hh(child));
                    let child_size = ((self.func)(child)).layout(
                        path,
                        Rect::from_origin_size(
                            (offset.x + width_sum, offset.y),
                            proposed_child_size,
                        ),
                        cx,
                        dcx,
                    );
                    sizes.push(child_size.size());
                    path.pop();

                    width_sum += child_size.width();
                }

                let mut max_height = 0.0;
                for size in &sizes {
                    max_height = size.height.max(max_height)
                }

                let mut x = 0.0;
                for c in 0..self.ids.len() {
                    path.push(hh(&self.ids[c]));
                    let child_size = sizes[c];

                    let child_offset = align_v(
                        RRect::from_origin_size(Point::ZERO, child_size, RectRadii::default()),
                        RRect::from_origin_size(
                            (x, 0.0),
                            (child_size.width, max_height),
                            RectRadii::default(),
                        ),
                        VAlignment::Middle,
                    );

                    cx.set_layout_offset(path, child_offset);

                    path.pop();

                    x += child_size.width;
                }

                Rect::from_origin_size(offset, Size::new(width_sum, max_height))
            }
            ListOrientation::Vertical => {
                let n = self.ids.len() as f64;
                let proposed_child_size = Size::new(sz.width(), sz.height() / n);
                let offset = sz.origin();

                let mut sizes = Vec::<Size>::new();
                sizes.reserve(self.ids.len());

                let mut height_sum = 0.0;
                for child in &self.ids {
                    path.push(hh(child));
                    let child_size = ((self.func)(child)).layout(
                        path,
                        Rect::from_origin_size(
                            (offset.x, offset.y + height_sum),
                            proposed_child_size,
                        ),
                        cx,
                        dcx,
                    );
                    sizes.push(child_size.size());
                    path.pop();

                    height_sum += child_size.height();
                }

                let mut max_width = 0.0;
                for size in &sizes {
                    max_width = size.width.max(max_width)
                }

                let mut y = height_sum;
                for c in 0..self.ids.len() {
                    path.push(hh(&self.ids[c]));
                    let child_size = sizes[c];

                    let child_offset = align_h(
                        RRect::from_origin_size(Point::ZERO, child_size, RectRadii::default()),
                        RRect::from_origin_size(
                            (0.0, y - child_size.height),
                            (max_width, child_size.height),
                            RectRadii::default(),
                        ),
                        HAlignment::Center,
                    );

                    cx.set_layout_offset(path, child_offset);
                    path.pop();

                    y -= child_size.height;
                }

                Rect::from_origin_size(offset, Size::new(max_width, height_sum))
            }
            ListOrientation::Z => {
                for child in &self.ids {
                    path.push(hh(child));
                    ((self.func)(child)).layout(path, sz, cx, dcx);
                    path.pop();
                }
                sz
            }
        }
    }

    fn dirty(&self, path: &mut IdPath, xform: Vec2, cx: &mut Context) {
        for child in &self.ids {
            path.push(hh(child));
            let offset = cx.get_layout(path).origin();
            let xf = xform + offset.to_vec2();
            ((self.func)(child)).dirty(path, xf, cx);
            path.pop();
        }
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, cx: &mut Context) -> Option<ViewId> {
        let mut hit = None;
        for child in &self.ids {
            path.push(hh(child));
            let offset = cx.get_layout(path).origin();

            if let Some(h) = ((self.func)(child)).hittest(path, (pt - offset).to_point(), cx) {
                hit = Some(h)
            }
            path.pop();
        }
        hit
    }

    fn commands(&self, path: &mut IdPath, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        for child in &self.ids {
            path.push(hh(child));
            ((self.func)(child)).commands(path, cx, cmds);
            path.pop();
        }
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(cx.view_id(path));
        for child in &self.ids {
            path.push(hh(child));
            map.push(cx.view_id(path));
            ((self.func)(child)).gc(path, cx, map);
            path.pop();
        }
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let mut builder = accesskit::NodeBuilder::new(accesskit::Role::List);

        let children: Vec<accesskit::NodeId> = self
            .ids
            .iter()
            .filter_map(|child| {
                path.push(hh(child));
                let node_id = ((self.func)(child)).access(path, cx, nodes);
                path.pop();
                node_id
            })
            .collect();

        builder.set_children(children);
        nodes.push((
            cx.view_id(path).access_id(),
            builder.build(&mut cx.access_node_classes),
        ));
        Some(cx.view_id(path).access_id())
    }
}

impl<ID, F> private::Sealed for List<ID, F> {}

/// Displays a list of items all of which are represented by the same View. See `examples/list.rs`.
///
/// `ids` is a Vec of items that implement Hash.
///
/// `f` is a function called to generate a View for each item.
///
/// For example:
///
/// ```no_run
/// # use rui::*;
/// rui(list(vec![1, 2, 3], |i| {
///     hstack((
///         circle(),
///         text(&format!("{:?}", i))
///     ))
/// }));
/// ```
pub fn list<ID: Hash, V: View, F: Fn(&ID) -> V + 'static>(ids: Vec<ID>, f: F) -> List<ID, F> {
    List {
        orientation: ListOrientation::Vertical,
        ids,
        func: f,
    }
}

pub fn hlist<ID: Hash, V: View, F: Fn(&ID) -> V + 'static>(ids: Vec<ID>, f: F) -> List<ID, F> {
    List {
        orientation: ListOrientation::Horizontal,
        ids,
        func: f,
    }
}

pub fn zlist<ID: Hash, V: View, F: Fn(&ID) -> V + 'static>(ids: Vec<ID>, f: F) -> List<ID, F> {
    List {
        orientation: ListOrientation::Z,
        ids,
        func: f,
    }
}
