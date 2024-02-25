use crate::*;

/// Region type cribbed from Druid.
#[derive(Clone, Debug)]
pub struct Region {
    rects: Vec<Rect>,
}

impl Region {
    /// The empty region.
    pub const EMPTY: Region = Region { rects: Vec::new() };

    /// Returns the collection of rectangles making up this region.
    #[inline]
    pub fn rects(&self) -> &[Rect] {
        &self.rects
    }

    /// Adds a rectangle to this region.
    pub fn add_rect(&mut self, rect: Rect) {
        if !rect.is_empty() {
            self.rects.push(rect);
        }
    }

    /// Replaces this region with a single rectangle.
    pub fn set_rect(&mut self, rect: Rect) {
        self.clear();
        self.add_rect(rect);
    }

    /// Sets this region to the empty region.
    pub fn clear(&mut self) {
        self.rects.clear();
    }

    /// Returns a rectangle containing this region.
    pub fn bounding_box(&self) -> Rect {
        if self.rects.is_empty() {
            Rect::default()
        } else {
            self.rects[1..]
                .iter()
                .fold(self.rects[0], |r, s| r.union(*s))
        }
    }

    /// Returns `true` if this region has a non-empty intersection with the given rectangle.
    pub fn intersects(&self, rect: Rect) -> bool {
        self.rects.iter().any(|r| !r.intersect(rect).is_empty())
    }

    /// Returns `true` if this region is empty.
    pub fn is_empty(&self) -> bool {
        // Note that we only ever add non-empty rects to self.rects.
        self.rects.is_empty()
    }

    /// Modifies this region by including everything in the other region.
    pub fn union_with(&mut self, other: &Region) {
        self.rects.extend_from_slice(&other.rects);
    }
}

impl std::ops::AddAssign<Vec2> for Region {
    fn add_assign(&mut self, rhs: Vec2) {
        for r in &mut self.rects {
            *r = r.translate(rhs)
        }
    }
}

impl std::ops::SubAssign<Vec2> for Region {
    fn sub_assign(&mut self, rhs: Vec2) {
        for r in &mut self.rects {
            *r = r.translate(-rhs)
        }
    }
}

impl From<Rect> for Region {
    fn from(rect: Rect) -> Region {
        Region { rects: vec![rect] }
    }
}
