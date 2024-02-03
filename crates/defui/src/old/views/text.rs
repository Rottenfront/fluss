use crate::*;

pub trait TextModifiers: View + Sized {
    fn font_size(self, size: f64) -> Text;
    fn color(self, color: Color) -> Text;
    fn font(self, font: FontId) -> Text;
}

/// Struct for `text`.
#[derive(Clone)]
pub struct Text {
    text: String,
    size: f64,
    color: Color,
    font: FontId,
}

impl Text {
    pub const DEFAULT_SIZE: u32 = 18;
    pub fn color(self, color: Color) -> Text {
        Text {
            text: self.text,
            size: self.size,
            color,
            font: self.font,
        }
    }

    pub fn font_size(self, size: f64) -> Text {
        Text {
            text: self.text,
            size,
            color: self.color,
            font: self.font,
        }
    }
}

impl View for Text {
    fn draw(&self, _path: &mut IdPath, args: &mut DrawArgs) {
        let vger = &mut args.vger;
        let origin = vger
            .state()
            .text_bounds(self.text.as_str(), None, self.font, self.size)
            .unwrap();
        let paint = vger
            .state()
            .create_fast_paint(Paint::Color(self.color))
            .unwrap();
        vger.draw_text(
            self.text.as_str(),
            0.0,
            0.0,
            None,
            self.size,
            self.font,
            paint,
        );
    }
    fn layout(&self, _path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        let size = (args.text_bounds)(self.text.as_str(), self.size, None, self.font).unwrap();
        [size.width, size.height].into()
    }
    fn hittest(&self, _path: &mut IdPath, _pt: LocalPoint, _cx: &mut Context) -> Option<ViewId> {
        None
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let aid = cx.view_id(path).access_id();
        let mut builder = accesskit::NodeBuilder::new(accesskit::Role::StaticText);
        builder.set_name(self.text.clone());
        nodes.push((aid, builder.build(&mut cx.access_node_classes)));
        Some(aid)
    }
}

impl TextModifiers for Text {
    fn font_size(self, size: f64) -> Self {
        Self {
            text: self.text,
            color: self.color,
            size,
            font: self.font,
        }
    }
    fn color(self, color: Color) -> Text {
        Text {
            text: self.text,
            size: self.size,
            color,
            font: self.font,
        }
    }
    fn font(self, font: FontId) -> Text {
        Text {
            text: self.text,
            size: self.size,
            color: self.color,
            font,
        }
    }
}

impl private::Sealed for Text {}

/// Shows a string as a label (not editable).
pub fn text(name: &str) -> Text {
    Text {
        text: String::from(name),
        size: Text::DEFAULT_SIZE as f64,
        color: TEXT_COLOR,
        font: crate::FALLBACK_SERIF_FONT,
    }
}
/*
impl<V> View for V
where
    V: std::fmt::Display + std::fmt::Debug + 'static,
{
    fn draw(&self, _path: &mut IdPath, args: &mut DrawArgs) {
        let txt = &format!("{}", self);
        let vger = &mut args.vger;
        let origin = vger.state().text_bounds(txt, None, DEFAULT_FONT);

        vger.save();
        vger.translate([-origin.x, -origin.y]);
        vger.text(txt, Text::DEFAULT_SIZE, TEXT_COLOR, None);
        vger.restore();
    }
    fn layout(&self, _path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        let txt = &format!("{}", self);
        (args.text_bounds)(txt, Text::DEFAULT_SIZE, None).size
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let aid = cx.view_id(path).access_id();
        let mut builder = accesskit::NodeBuilder::new(accesskit::Role::StaticText);
        builder.set_name(format!("{}", self));
        nodes.push((aid, builder.build(&mut cx.access_node_classes)));
        Some(aid)
    }
}

impl<V> TextModifiers for V
where
    V: std::fmt::Display + std::fmt::Debug + 'static,
{
    fn font_size(self, size: u32) -> Text {
        Text {
            text: format!("{}", self),
            size,
            color: TEXT_COLOR,
            font: FALLBACK_SERIF_FONT
        }
    }
    fn color(self, color: Color) -> Text {
        Text {
            text: format!("{}", self),
            size: Text::DEFAULT_SIZE,
            color,
            font: FALLBACK_SERIF_FONT
        }
    }
}

impl<V> private::Sealed for V where V: std::fmt::Display {}
*/
