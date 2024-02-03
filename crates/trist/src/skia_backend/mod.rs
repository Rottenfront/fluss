#[cfg(feature = "skia-gl")]
mod gl_backend;

#[cfg(feature = "skia-gl")]
pub use gl_backend::DrawerEnv;

use kurbo::Shape;
use skia_safe::{
    font_style::{Slant as SSlant, Weight as SWeight, Width as SWidth},
    Font as SFont, FontMgr as SFontMgr, FontStyle as SFontStyle, Paint as SPaint, Path as SPath,
};
use std::collections::HashMap;

use crate::{
    DrawerError, Font, FontId, Paint, PaintId, TristDrawer, TristDrawerState, Weight, Width,
};

pub struct DrawerState {
    fast_paints: HashMap<usize, SPaint>,
    static_paints: HashMap<usize, SPaint>,
    fonts: HashMap<usize, SFont>,
    last_fast_paint_id: usize,
    last_static_paint_id: usize,
    last_font_id: usize,
}

impl DrawerState {
    pub fn new() -> Self {
        Self {
            fast_paints: HashMap::new(),
            static_paints: HashMap::new(),
            last_fast_paint_id: 0,
            last_static_paint_id: 0,
            fonts: HashMap::new(),
            last_font_id: 0,
        }
    }

    pub fn clear(&mut self) {
        self.fast_paints.clear();
    }

    fn get_paint(&self, paint: PaintId) -> Result<&SPaint, DrawerError> {
        if paint.is_static {
            match self.static_paints.get(&paint.id) {
                None => Err(DrawerError::NoPaint(paint)),
                Some(paint) => Ok(paint),
            }
        } else {
            match self.fast_paints.get(&paint.id) {
                None => Err(DrawerError::NoPaint(paint)),
                Some(paint) => Ok(paint),
            }
        }
    }
}

fn get_skia_font_weight(weight: Weight) -> SWeight {
    match weight {
        Weight::Invisible => SWeight::INVISIBLE,
        Weight::Thin => SWeight::THIN,
        Weight::ExtraLight => SWeight::EXTRA_LIGHT,
        Weight::Light => SWeight::LIGHT,
        Weight::Normal => SWeight::NORMAL,
        Weight::Medium => SWeight::MEDIUM,
        Weight::Semibold => SWeight::SEMI_BOLD,
        Weight::Bold => SWeight::BOLD,
        Weight::ExtraBold => SWeight::EXTRA_BOLD,
        Weight::Black => SWeight::BLACK,
        Weight::ExtraBlack => SWeight::EXTRA_BLACK,
    }
}

fn get_skia_font_width(width: Width) -> SWidth {
    match width {
        Width::UltraCondensed => SWidth::ULTRA_CONDENSED,
        Width::ExtraCondensed => SWidth::EXTRA_CONDENSED,
        Width::Condensed => SWidth::CONDENSED,
        Width::SemiCondensed => SWidth::SEMI_CONDENSED,
        Width::Normal => SWidth::NORMAL,
        Width::SemiExpanded => SWidth::SEMI_EXPANDED,
        Width::Expanded => SWidth::EXPANDED,
        Width::ExtraExpanded => SWidth::EXTRA_EXPANDED,
        Width::UltraExpanded => SWidth::ULTRA_EXPANDED,
    }
}

impl TristDrawerState for DrawerState {
    fn create_fast_paint(&mut self, paint: Paint) -> Result<PaintId, DrawerError> {
        self.fast_paints
            .insert(self.last_fast_paint_id, get_skia_paint(paint));
        self.last_fast_paint_id += 1;
        Ok(PaintId {
            id: self.last_fast_paint_id - 1,
            is_static: false,
        })
    }

    fn create_static_paint(&mut self, paint: Paint) -> Result<PaintId, DrawerError> {
        self.static_paints
            .insert(self.last_fast_paint_id, get_skia_paint(paint));
        self.last_static_paint_id += 1;
        Ok(PaintId {
            id: self.last_static_paint_id - 1,
            is_static: true,
        })
    }

    #[allow(unused_must_use)]
    fn remove_static_paint(&mut self, paint: PaintId) -> Result<(), DrawerError> {
        match self.static_paints.remove(&paint.id) {
            None => Err(DrawerError::NoPaint(paint)),
            Some(_) => Ok(()),
        }
    }

    fn create_font(&mut self, font: Font) -> Result<FontId, DrawerError> {
        let font = SFont::new(
            SFontMgr::new()
                .match_family_style(
                    &font.name,
                    SFontStyle::new(
                        get_skia_font_weight(font.weight),
                        get_skia_font_width(font.width),
                        SSlant::Upright,
                    ),
                )
                .unwrap(),
            font.size as f32,
        );
        self.fonts.insert(self.last_font_id, font);
        self.last_font_id += 1;
        Ok(FontId {
            id: self.last_font_id - 1,
        })
    }

    #[allow(unused_must_use)]
    fn remove_font(&mut self, font: FontId) -> Result<(), DrawerError> {
        match self.fonts.remove(&font.id) {
            None => Err(DrawerError::NoFont(font)),
            Some(_) => Ok(()),
        }
    }

    fn text_bounds(
        &self,
        text: &str,
        max_width: Option<f64>,
        font_id: FontId,
        size: f64,
    ) -> Result<kurbo::Size, DrawerError> {
        let font = match self.fonts.get(&font_id.id) {
            None => return Err(DrawerError::NoFont(font_id)),
            Some(font) => font,
        };
        if let Some(font) = &font.with_size(size as f32) {
            let (wraps, width) = get_text_wraps(text, font, max_width);
            Ok(kurbo::Size::new(
                width as _,
                font.metrics().0 as f64 * (1.0 + wraps.len() as f64),
            ))
        } else {
            Err(DrawerError::NoFont(font_id))
        }
    }
}

pub struct Drawer<'a> {
    canvas: &'a skia_safe::Canvas,
    state: &'a mut DrawerState,
    current_translate: (f64, f64),
}

fn get_skia_path(shape: &impl Shape) -> SPath {
    let mut path = SPath::new();
    let kurbo_path = shape.to_path(0.1); // TODO: make this understandable (wtf is tolerance)
    for p in kurbo_path.elements() {
        match p {
            kurbo::PathEl::MoveTo(pt) => path.move_to((pt.x as f32, pt.y as f32)),
            kurbo::PathEl::LineTo(pt) => path.line_to((pt.x as f32, pt.y as f32)),
            kurbo::PathEl::QuadTo(pt1, pt2) => {
                path.quad_to((pt1.x as f32, pt1.y as f32), (pt2.x as f32, pt2.y as f32))
            }
            kurbo::PathEl::CurveTo(pt1, pt2, pt3) => path.cubic_to(
                (pt1.x as f32, pt1.y as f32),
                (pt2.x as f32, pt2.y as f32),
                (pt3.x as f32, pt3.y as f32),
            ),
            kurbo::PathEl::ClosePath => path.close(),
        };
    }
    path
}

fn get_skia_paint(paint: Paint) -> SPaint {
    match paint {
        Paint::Color(color) => SPaint::new(
            skia_safe::Color4f::new(color.r, color.g, color.b, color.a),
            None,
        ),
    }
}

fn get_text_wraps(text: &str, font: &SFont, max_width: Option<f64>) -> (Vec<usize>, f64) {
    let chars = text.chars();
    let mut widths = Vec::new();
    let mut buf = [0.0];
    for c in chars {
        font.get_widths(&[c as _], &mut buf);
        widths.push(buf[0] as f64);
    }

    match max_width {
        None => {
            let mut width = 0.0;
            for i in 0..text.len() {
                width += widths[i];
            }

            (vec![], width)
        }
        Some(max_width) => {
            let mut wraps = Vec::new();
            let mut max_blob_width = 0.0;
            let mut width = 0.0;
            for i in 0..text.len() {
                width += widths[i];
                if width > max_width {
                    wraps.push(i);
                    width = 0.0;
                    max_blob_width = max_width.max(max_width);
                }
            }

            let width = max_blob_width.max(width);
            (wraps, width)
        }
    }
}

impl<'a> Drawer<'a> {
    pub fn new(canvas: &'a skia_safe::Canvas, state: &'a mut DrawerState) -> Self {
        Self {
            canvas,
            state,
            current_translate: (0.0, 0.0),
        }
    }
}

impl<'a> TristDrawer<DrawerState> for Drawer<'a> {
    #[allow(unused_must_use)]
    fn save(&mut self) {
        self.canvas.save();
    }

    #[allow(unused_must_use)]
    fn restore(&mut self) {
        self.canvas.restore();
    }

    #[allow(unused_must_use)]
    fn translate(&mut self, offset: kurbo::Vec2) {
        self.current_translate.0 += offset.x as f64;
        self.current_translate.1 += offset.y as f64;
        self.canvas.translate((offset.x as i32, offset.y as i32));
    }

    #[allow(unused_must_use)]
    fn clip_shape(&mut self, shape: &impl Shape) {
        self.canvas
            .clip_path(&get_skia_path(shape), skia_safe::ClipOp::Intersect, true);
    }

    #[allow(unused_must_use)]
    fn draw_shape(&mut self, shape: &impl Shape, paint: PaintId) {
        match self.state.get_paint(paint) {
            Err(e) => {}
            Ok(paint) => {
                self.canvas.draw_path(&get_skia_path(shape), paint);
            }
        }
    }

    fn draw_text(
        &mut self,
        text: &str,
        x: f64,
        y: f64,
        max_width: Option<f64>,
        size: f64,
        font: FontId,
        paint: PaintId,
    ) {
        match self.state.fonts.get(&font.id) {
            None => {}
            Some(sk_font) => {
                self.canvas.draw_text_blob(
                    match skia_safe::TextBlob::new(
                        text,
                        match &sk_font.with_size(size as f32) {
                            None => return,
                            Some(font) => font,
                        },
                    ) {
                        None => return,
                        Some(blob) => blob,
                    },
                    (x as f32, y as f32),
                    match self.state.get_paint(paint) {
                        Err(_) => return,
                        Ok(paint) => paint,
                    },
                );
            }
        }
    }

    fn state(&mut self) -> &mut DrawerState {
        self.state
    }

    fn current_transform(&self) -> kurbo::Vec2 {
        self.current_translate.into()
    }
}
