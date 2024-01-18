use crate::canvas::Drawer;

use super::tab_system::TabSystem;
use skia_safe::Rect;

pub enum SplitDirection {
    Vertical,
    Horizontal,
}

pub struct Split {
    pub direction: SplitDirection,
    pub main_item: Box<SplitItem>,
    pub next_item: Option<Box<SplitItem>>,
    // between 0 and 1 - percentage of main item width or height depending on the direction
    pub fraction: f32,
    // true is enabled next item, false if enabled main item
    pub enabled: bool,
}

impl Split {
    pub fn focused_tab_system_mut(&mut self) -> &mut TabSystem {
        match &mut self.next_item {
            None => match &mut *self.main_item {
                SplitItem::Split(split) => split.focused_tab_system_mut(),
                SplitItem::TabSystem(ts) => ts,
            },
            Some(sp) => {
                if self.enabled {
                    match &mut **sp {
                        SplitItem::Split(split) => split.focused_tab_system_mut(),
                        SplitItem::TabSystem(ts) => ts,
                    }
                } else {
                    match &mut *self.main_item {
                        SplitItem::Split(split) => split.focused_tab_system_mut(),
                        SplitItem::TabSystem(ts) => ts,
                    }
                }
            }
        }
    }
    pub fn focused_tab_system(&self) -> &TabSystem {
        match &self.next_item {
            None => match &*self.main_item {
                SplitItem::Split(split) => split.focused_tab_system(),
                SplitItem::TabSystem(ts) => ts,
            },
            Some(sp) => {
                if self.enabled {
                    match &**sp {
                        SplitItem::Split(split) => split.focused_tab_system(),
                        SplitItem::TabSystem(ts) => ts,
                    }
                } else {
                    match &*self.main_item {
                        SplitItem::Split(split) => split.focused_tab_system(),
                        SplitItem::TabSystem(ts) => ts,
                    }
                }
            }
        }
    }

    pub fn draw(&self, drawer: &Drawer, rect: Rect) {
        let (x1, y1, x2, y2) = (rect.left, rect.top, rect.right, rect.bottom);
        match &self.next_item {
            None => self.main_item.draw(drawer, rect),
            Some(next) => match self.direction {
                SplitDirection::Vertical => {
                    let middle_y = y1 * self.fraction + y2 * (1.0 - self.fraction);
                    self.main_item.draw(
                        drawer,
                        Rect {
                            left: x1,
                            top: y1,
                            right: x2,
                            bottom: middle_y,
                        },
                    );
                    next.draw(
                        drawer,
                        Rect {
                            left: x1,
                            top: middle_y,
                            right: x2,
                            bottom: y2,
                        },
                    );
                }
                SplitDirection::Horizontal => {
                    let middle_x = x1 * self.fraction + x2 * (1.0 - self.fraction);
                    self.main_item.draw(
                        drawer,
                        Rect {
                            left: x1,
                            top: y1,
                            right: middle_x,
                            bottom: y2,
                        },
                    );
                    next.draw(
                        drawer,
                        Rect {
                            left: middle_x,
                            top: y1,
                            right: x2,
                            bottom: y2,
                        },
                    );
                }
            },
        }
    }
}

pub enum SplitItem {
    Split(Split),
    TabSystem(TabSystem),
}

impl SplitItem {
    pub fn draw(&self, drawer: &Drawer, rect: Rect) {
        match self {
            Self::Split(sp) => sp.draw(drawer, rect),
            Self::TabSystem(ts) => ts.draw(drawer, rect),
        }
    }
}
