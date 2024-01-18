use skia_safe::{Canvas, Font};
use std::collections::HashMap;

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct FontId(pub usize);

pub struct Drawer<'a> {
    pub canvas: &'a Canvas,
    pub fonts: HashMap<FontId, Font>,
}
