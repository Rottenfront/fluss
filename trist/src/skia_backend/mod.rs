#[cfg(feature = "skia-gl")]
mod gl_backend;

#[cfg(feature = "skia-gl")]
pub use gl_backend::SkiaEnv;

use kurbo::Shape;
use skia_safe::{
    font_style::{Slant as SSlant, Weight as SWeight, Width as SWidth},
    Font as SFont, FontMgr as SFontMgr, FontStyle as SFontStyle, Paint as SPaint, Path as SPath,
};
use std::collections::HashMap;

use crate::{Drawer, DrawerError, DrawerState, Font, FontId, Paint, PaintId, Weight, Width};

pub struct SkiaDrawerState {
    fast_paints: HashMap<usize, SPaint>,
    static_paints: HashMap<usize, SPaint>,
    fonts: HashMap<usize, SFont>,
    last_fast_paint_id: usize,
    last_static_paint_id: usize,
    last_font_id: usize,
}

impl SkiaDrawerState {
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

impl DrawerState for SkiaDrawerState {
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
            font.size,
        );
        self.fonts.insert(self.last_font_id, font);
        self.last_font_id += 1;
        Ok(FontId {
            id: self.last_font_id - 1,
        })
    }

    fn remove_font(&mut self, font: FontId) -> Result<(), DrawerError> {
        match self.fonts.remove(&font.id) {
            None => Err(DrawerError::NoFont(font)),
            Some(_) => Ok(()),
        }
    }
}

pub struct SkiaDrawer<'a> {
    canvas: &'a skia_safe::Canvas,
    state: &'a mut SkiaDrawerState,
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

impl<'a> SkiaDrawer<'a> {
    pub fn new(canvas: &'a skia_safe::Canvas, state: &'a mut SkiaDrawerState) -> Self {
        Self { canvas, state }
    }
}

impl<'a> Drawer<SkiaDrawerState> for SkiaDrawer<'a> {
    fn save(&mut self) -> Result<(), DrawerError> {
        self.canvas.save();
        Ok(())
    }

    fn restore(&mut self) -> Result<(), DrawerError> {
        self.canvas.restore();
        Ok(())
    }

    fn translate(&mut self, point: kurbo::Point) -> Result<(), DrawerError> {
        self.canvas.translate((point.x as i32, point.y as i32));
        Ok(())
    }

    fn clip_shape(&mut self, shape: &impl Shape) -> Result<(), DrawerError> {
        self.canvas
            .clip_path(&get_skia_path(shape), skia_safe::ClipOp::Intersect, true);
        Ok(())
    }

    fn draw_shape(&mut self, shape: &impl Shape, paint: PaintId) -> Result<(), DrawerError> {
        match self.state.get_paint(paint) {
            Err(e) => Err(e),
            Ok(paint) => {
                self.canvas.draw_path(&get_skia_path(shape), paint);
                Ok(())
            }
        }
    }

    fn draw_text(
        &mut self,
        text: String,
        x: f32,
        y: f32,
        font: FontId,
        paint: PaintId,
    ) -> Result<(), DrawerError> {
        match self.state.fonts.get(&font.id) {
            None => Err(DrawerError::NoFont(font)),
            Some(sk_font) => {
                self.canvas.draw_text_blob(
                    match skia_safe::TextBlob::new(text, sk_font) {
                        None => return Err(DrawerError::NoFont(font)),
                        Some(blob) => blob,
                    },
                    (x, y),
                    match self.state.get_paint(paint) {
                        Err(e) => return Err(e),
                        Ok(paint) => paint,
                    },
                );
                Ok(())
            }
        }
    }

    fn state(&mut self) -> &mut SkiaDrawerState {
        self.state
    }
}