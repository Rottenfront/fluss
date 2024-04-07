use crate::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Font {
    family: FontFamily,
    weight: FontWeight,
    style: FontStyle,
    size: f64,
}

impl Font {
    pub const SYSTEM: Self = Self {
        family: FontFamily::SYSTEM_UI,
        weight: FontWeight::NORMAL,
        style: FontStyle::Regular,
        size: 14.0,
    };

    pub const SANS_SERIF: Self = Self {
        family: FontFamily::SANS_SERIF,
        weight: FontWeight::NORMAL,
        style: FontStyle::Regular,
        size: 14.0,
    };

    pub const SERIF: Self = Self {
        family: FontFamily::SERIF,
        weight: FontWeight::NORMAL,
        style: FontStyle::Regular,
        size: 14.0,
    };

    pub const MONOSPACE: Self = Self {
        family: FontFamily::SANS_SERIF,
        weight: FontWeight::NORMAL,
        style: FontStyle::Regular,
        size: 14.0,
    };

    pub fn new(family: FontFamily) -> Self {
        Self {
            family,
            weight: FontWeight::NORMAL,
            style: FontStyle::Regular,
            size: 14.0,
        }
    }

    pub fn with_style(self, style: FontStyle) -> Self {
        Self {
            family: self.family,
            weight: self.weight,
            style,
            size: self.size,
        }
    }

    pub fn with_weight(self, weight: FontWeight) -> Self {
        Self {
            family: self.family,
            weight,
            style: self.style,
            size: self.size,
        }
    }

    pub fn with_size(self, size: f64) -> Self {
        Self {
            family: self.family,
            weight: self.weight,
            style: self.style,
            size,
        }
    }
}

pub struct TextView {
    id: ViewId,
    text: fn() -> String,
    color: Binding<Color>,
    font: Binding<Font>,
}

impl TextView {
    pub fn new(text: fn() -> String, color: Binding<Color>, font: Binding<Font>) -> Self {
        Self {
            id: new_id(),
            color,
            text,
            font,
        }
    }
}

impl View for TextView {
    fn draw(&self, draw_ctx: DrawContext) {
        let DrawContext {
            drawer,
            size: max_size,
            ctx,
        } = draw_ctx;
        let offset = drawer.current_transform();
        let text = (self.text)();
        let font = self.font.get();
        let color = self.color.get();
        let layout = drawer
            .text()
            .new_text_layout(text)
            .default_attribute(TextAttribute::FontFamily(font.family))
            .default_attribute(TextAttribute::FontSize(font.size))
            .default_attribute(TextAttribute::Weight(font.weight))
            .default_attribute(TextAttribute::Style(font.style))
            .text_color(color)
            .build()
            .unwrap();
        let size = layout.size();
        let size = Size::new(size.width, size.height.min(max_size.height));
        self.update_layout(Layout::new(offset, size), ctx);
        drawer.draw_text(&layout, (0.0, 0.0));
    }

    fn get_id(&self) -> ViewId {
        self.id
    }

    fn process_event(&mut self, _event: &Event, _ctx: &mut Context) -> bool {
        false
    }

    fn get_min_size(&self, drawer: &mut Piet, _ctx: &mut Context) -> Size {
        let text = (self.text)();
        let font = self.font.get();
        drawer
            .text()
            .new_text_layout(text)
            .default_attribute(TextAttribute::FontFamily(font.family))
            .default_attribute(TextAttribute::FontSize(font.size))
            .default_attribute(TextAttribute::Style(font.style))
            .default_attribute(TextAttribute::Weight(font.weight))
            .build()
            .unwrap()
            .size()
    }

    fn is_flexible(&self) -> bool {
        true
    }
}
