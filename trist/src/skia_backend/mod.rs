#[cfg(feature = "opengl")]
mod gl_backend;
#[cfg(feature = "opengl")]
pub use gl_backend::DrawerEnv;
#[cfg(feature = "opengl")]
use skia_gl::{
    font_style::{Slant as SSlant, Weight as SWeight, Width as SWidth},
    Canvas as SCanvas, ClipOp as SClipOp, Color4f as SColor4f, Font as SFont, FontMgr as SFontMgr,
    FontStyle as SFontStyle, Paint as SPaint, Path as SPath, RRect as SRRect, Rect as SRect,
    TextBlob as STextBlob, Vector as SVector,
};

#[cfg(feature = "opengl-linux")]
mod gl_linux_backend;
#[cfg(feature = "opengl-linux")]
pub use gl_linux_backend::DrawerEnv;
#[cfg(feature = "opengl-linux")]
use skia_gl_linux::{
    font_style::{Slant as SSlant, Weight as SWeight, Width as SWidth},
    Canvas as SCanvas, ClipOp as SClipOp, Color4f as SColor4f, Font as SFont, FontMgr as SFontMgr,
    FontStyle as SFontStyle, Paint as SPaint, Path as SPath, RRect as SRRect, Rect as SRect,
    TextBlob as STextBlob, Vector as SVector,
};

#[cfg(feature = "metal")]
mod metal_backend;
#[cfg(feature = "metal")]
pub use metal_backend::DrawerEnv;
#[cfg(feature = "metal")]
use skia_metal::{
    font_style::{Slant as SSlant, Weight as SWeight, Width as SWidth},
    Canvas as SCanvas, ClipOp as SClipOp, Color4f as SColor4f, Font as SFont, FontMgr as SFontMgr,
    FontStyle as SFontStyle, Paint as SPaint, Path as SPath, RRect as SRRect, Rect as SRect,
    TextBlob as STextBlob, Vector as SVector,
};

#[cfg(feature = "directx")]
mod d3d_backend;
#[cfg(feature = "directx")]
pub use d3d_backend::DrawerEnv;
#[cfg(feature = "directx")]
use skia_d3d::{
    font_style::{Slant as SSlant, Weight as SWeight, Width as SWidth},
    Canvas as SCanvas, ClipOp as SClipOp, Color4f as SColor4f, Font as SFont, FontMgr as SFontMgr,
    FontStyle as SFontStyle, Paint as SPaint, Path as SPath, RRect as SRRect, Rect as SRect,
    TextBlob as STextBlob, Vector as SVector,
};

#[cfg(feature = "vulkan")]
mod vulkan_backend;
#[cfg(feature = "vulkan")]
use skia_vulkan::{
    font_style::{Slant as SSlant, Weight as SWeight, Width as SWidth},
    Canvas as SCanvas, ClipOp as SClipOp, Color4f as SColor4f, Font as SFont, FontMgr as SFontMgr,
    FontStyle as SFontStyle, Paint as SPaint, Path as SPath, RRect as SRRect, Rect as SRect,
    TextBlob as STextBlob, Vector as SVector,
};
#[cfg(feature = "vulkan")]
pub use vulkan_backend::DrawerEnv;

use std::collections::HashMap;

use crate::*;

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
        let mut state = Self {
            fast_paints: HashMap::new(),
            static_paints: HashMap::new(),
            last_fast_paint_id: 0,
            last_static_paint_id: 0,
            fonts: HashMap::new(),
            last_font_id: FALLBACK_SERIF_FONT.id.max(FALLBACK_MONOSPACE_FONT.id) + 1,
        };

        state.fonts.insert(
            FALLBACK_SERIF_FONT.id,
            SFont::new(
                SFontMgr::new()
                    .match_family_style(
                        "sans",
                        SFontStyle::new(SWeight::NORMAL, SWidth::NORMAL, SSlant::Upright),
                    )
                    .unwrap(),
                13.0,
            ),
        );

        state.fonts.insert(
            FALLBACK_MONOSPACE_FONT.id,
            SFont::new(
                SFontMgr::new()
                    .match_family_style(
                        "monospace",
                        SFontStyle::new(SWeight::NORMAL, SWidth::NORMAL, SSlant::Upright),
                    )
                    .unwrap(),
                13.0,
            ),
        );

        state
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
    ) -> Result<gcl::Size, DrawerError> {
        let font = match self.fonts.get(&font_id.id) {
            None => return Err(DrawerError::NoFont(font_id)),
            Some(font) => font,
        };
        if let Some(font) = &font.with_size(size as f32) {
            let (wraps, width) = get_text_wraps(text, font, max_width);
            Ok(gcl::Size::new(
                width as _,
                font.metrics().0 as f64 * (1.0 + wraps.len() as f64),
            ))
        } else {
            Err(DrawerError::NoFont(font_id))
        }
    }
}

pub struct Drawer<'a, 'b> {
    canvas: &'a SCanvas,
    state: &'b mut DrawerState,
    current_translate: TranslateScale,
}

fn get_skia_path(shape: &impl Shape) -> SPath {
    let mut path = SPath::new();
    let gcl_path = shape.to_path(0.1); // TODO: make this understandable (wtf is tolerance)
    for p in gcl_path.elements() {
        match p {
            gcl::PathEl::MoveTo(pt) => path.move_to((pt.x as f32, pt.y as f32)),
            gcl::PathEl::LineTo(pt) => path.line_to((pt.x as f32, pt.y as f32)),
            gcl::PathEl::QuadTo(pt1, pt2) => {
                path.quad_to((pt1.x as f32, pt1.y as f32), (pt2.x as f32, pt2.y as f32))
            }
            gcl::PathEl::CurveTo(pt1, pt2, pt3) => path.cubic_to(
                (pt1.x as f32, pt1.y as f32),
                (pt2.x as f32, pt2.y as f32),
                (pt3.x as f32, pt3.y as f32),
            ),
            gcl::PathEl::ClosePath => path.close(),
        };
    }
    path
}

fn get_skia_rect(rect: &RRect) -> SRRect {
    let base = &rect.rect();
    let radii = &rect.radii();
    SRRect::new_rect_radii(
        SRect::new(
            base.x0 as f32,
            base.y0 as f32,
            base.x1 as f32,
            base.y1 as f32,
        ),
        &[
            SVector::new(radii.top_left as f32, radii.top_left as f32),
            SVector::new(radii.top_right as f32, radii.top_right as f32),
            SVector::new(radii.bottom_right as f32, radii.bottom_right as f32),
            SVector::new(radii.bottom_left as f32, radii.bottom_left as f32),
        ],
    )
}

fn get_skia_paint(paint: Paint) -> SPaint {
    match paint {
        Paint::Color(color) => SPaint::new(SColor4f::new(color.r, color.g, color.b, color.a), None),
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

impl<'a, 'b> Drawer<'a, 'b> {
    pub fn new(canvas: &'a SCanvas, state: &'b mut DrawerState) -> Self {
        Self {
            canvas,
            state,
            current_translate: Default::default(),
        }
    }
}

impl<'a, 'b> TristDrawer<DrawerState> for Drawer<'a, 'b> {
    fn clear(&mut self, color: Color) {
        self.canvas
            .clear(SColor4f::new(color.r, color.g, color.b, color.a));
    }

    fn save(&mut self) {
        self.canvas.save();
    }

    fn restore(&mut self) {
        self.canvas.restore();
    }

    fn translate(&mut self, translate: TranslateScale) {
        self.current_translate *= translate;
        self.canvas.translate((
            translate.translation.x as f32,
            translate.translation.y as f32,
        ));
        self.canvas
            .scale((translate.scale_x as f32, translate.scale_y as f32));
    }

    fn clip_shape(&mut self, shape: &impl Shape) {
        self.canvas
            .clip_path(&get_skia_path(shape), SClipOp::Intersect, true);
    }

    fn clip_rect(&mut self, rect: &RRect) {
        self.canvas
            .clip_rrect(&get_skia_rect(rect), SClipOp::Intersect, true);
    }

    fn draw_shape(&mut self, shape: &impl Shape, paint: PaintId) {
        match self.state.get_paint(paint) {
            Err(_) => {}
            Ok(paint) => {
                self.canvas.draw_path(&get_skia_path(shape), paint);
            }
        }
    }

    fn draw_rect(&mut self, rect: &RRect, paint: PaintId) {
        match self.state.get_paint(paint) {
            Err(_) => {}
            Ok(paint) => {
                let rect = get_skia_rect(rect);
                self.canvas.draw_rrect(&rect, paint);
            }
        }
    }

    fn draw_circle(&mut self, center: gcl::Point, radius: f64, paint: PaintId) {
        match self.state.get_paint(paint) {
            Err(_) => {}
            Ok(paint) => {
                self.canvas
                    .draw_circle((center.x as f32, center.y as f32), radius as f32, paint);
            }
        }
    }

    fn draw_text(
        &mut self,
        text: &str,
        pos: Vec2,
        _max_width: Option<f64>,
        size: f64,
        font: FontId,
        paint: PaintId,
    ) {
        match self.state.fonts.get(&font.id) {
            None => {}
            Some(sk_font) => {
                self.canvas.draw_text_blob(
                    match STextBlob::new(
                        text,
                        match &sk_font.with_size(size as f32) {
                            None => return,
                            Some(font) => font,
                        },
                    ) {
                        None => return,
                        Some(blob) => blob,
                    },
                    (pos.x as f32, pos.y as f32),
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

    fn current_transform(&self) -> TranslateScale {
        self.current_translate
    }
}
