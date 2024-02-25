use crate::*;

pub trait TextModifiers: Sized {
    fn font_size(self, size: f64) -> Text;
    fn color(self, color: Color) -> Text;
    fn font(self, font: FontId) -> Text;
    fn max_width(self, font: Option<f64>) -> Text;
}

/// Struct for `text`.
#[derive(Clone)]
pub struct Text {
    text: String,
    size: f64,
    color: Color,
    font: FontId,
    max_width: Option<f64>,
    // private variables
    pos: FBinding<Vec2>,
    draw_max_width: FBinding<f64>,
}

impl Text {
    pub const DEFAULT_SIZE: f64 = 18.0;
    pub const DEFAULT_FONT: FontId = FALLBACK_SERIF_FONT;
    pub fn color(self, color: Color) -> Text {
        Text {
            text: self.text,
            size: self.size,
            color,
            font: self.font,
            max_width: self.max_width,
            pos: self.pos,
            draw_max_width: self.draw_max_width,
        }
    }
}

impl View for Text {
    fn draw(&self, _path: &mut IdPath, _cx: &mut Context, vger: &mut Drawer) {
        let paint = vger.color_paint(self.color).unwrap();
        println!("{}", self.pos.get());
        vger.draw_text(
            self.text.as_str(),
            self.pos.get(),
            Some(self.draw_max_width.get()),
            self.size,
            self.font,
            paint,
        );
    }
    fn layout(
        &self,
        _path: &mut IdPath,
        sz: Rect,
        _cx: &mut Context,
        dcx: &mut DrawerState,
    ) -> Rect {
        self.pos.set(sz.origin().to_vec2());
        self.draw_max_width.set(match self.max_width {
            Some(width) => width.min(sz.width()),
            None => sz.width(),
        });
        sz.with_size(
            dcx.text_bounds(
                self.text.as_str(),
                Some(self.draw_max_width.get()),
                self.font,
                self.size,
            )
            .unwrap(),
        )
    }
    fn hittest(&self, _path: &mut IdPath, _pt: Point, _cx: &mut Context) -> Option<ViewId> {
        None
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let aid = cx.view_id(path).access_id();
        let mut builder = accesskit::NodeBuilder::new(accesskit::Role::LabelText);
        builder.set_name(self.text.clone());
        nodes.push((aid, builder.build(&mut cx.access_node_classes)));
        Some(aid)
    }
}

impl TextModifiers for Text {
    fn font_size(self, size: f64) -> Text {
        Text {
            text: self.text,
            color: self.color,
            size,
            font: self.font,
            max_width: self.max_width,
            pos: self.pos,
            draw_max_width: self.draw_max_width,
        }
    }
    fn color(self, color: Color) -> Text {
        Text {
            text: self.text,
            size: self.size,
            color,
            font: self.font,
            max_width: self.max_width,
            pos: self.pos,
            draw_max_width: self.draw_max_width,
        }
    }

    fn font(self, font: FontId) -> Text {
        Text {
            text: self.text,
            size: self.size,
            color: self.color,
            font,
            max_width: self.max_width,
            pos: self.pos,
            draw_max_width: self.draw_max_width,
        }
    }

    fn max_width(self, max_width: Option<f64>) -> Text {
        Text {
            text: self.text,
            size: self.size,
            color: self.color,
            font: self.font,
            max_width,
            pos: self.pos,
            draw_max_width: self.draw_max_width,
        }
    }
}

impl private::Sealed for Text {}

/// Shows a string as a label (not editable).
pub fn text(name: &str) -> Text {
    Text {
        text: String::from(name),
        size: Text::DEFAULT_SIZE,
        color: TEXT_COLOR,
        font: Text::DEFAULT_FONT,
        max_width: None,
        pos: fbind((0.0, 0.0).into()),
        draw_max_width: fbind(0.0),
    }
}

impl<V> TextModifiers for V
where
    V: std::fmt::Display + std::fmt::Debug + 'static,
{
    fn font_size(self, size: f64) -> Text {
        Text {
            text: format!("{}", self),
            size,
            color: TEXT_COLOR,
            font: Text::DEFAULT_FONT,
            max_width: None,
            pos: fbind((0.0, 0.0).into()),
            draw_max_width: fbind(0.0),
        }
    }
    fn color(self, color: Color) -> Text {
        Text {
            text: format!("{}", self),
            size: Text::DEFAULT_SIZE,
            color,
            font: Text::DEFAULT_FONT,
            max_width: None,
            pos: fbind((0.0, 0.0).into()),
            draw_max_width: fbind(0.0),
        }
    }
    fn font(self, font: FontId) -> Text {
        Text {
            text: format!("{}", self),
            size: Text::DEFAULT_SIZE,
            color: TEXT_COLOR,
            font,
            max_width: None,
            pos: fbind((0.0, 0.0).into()),
            draw_max_width: fbind(0.0),
        }
    }

    fn max_width(self, max_width: Option<f64>) -> Text {
        Text {
            text: format!("{}", self),
            size: Text::DEFAULT_SIZE,
            color: TEXT_COLOR,
            font: Text::DEFAULT_FONT,
            max_width,
            pos: fbind((0.0, 0.0).into()),
            draw_max_width: fbind(0.0),
        }
    }
}

impl<V> private::Sealed for V where V: std::fmt::Display {}
