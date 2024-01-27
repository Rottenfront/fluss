use crate::*;

const SLIDER_WIDTH: f64 = 4.0;
const SLIDER_THUMB_RADIUS: f64 = 10.0;

#[derive(Clone, Copy)]
pub struct SliderOptions {
    thumb: Color,
}

impl Default for SliderOptions {
    fn default() -> Self {
        Self {
            thumb: AZURE_HIGHLIGHT,
        }
    }
}

pub trait SliderMods: View + Sized {
    fn thumb_color(self, color: Color) -> Self;
}

/// Horizontal slider built from other Views.
pub fn hslider(value: impl Binding<f64>) -> impl SliderMods {
    modview(move |opts: SliderOptions, _| {
        state(
            || 0.0,
            move |width, cx| {
                let w = cx[width];
                canvas(move |cx, sz, vger| {
                    let c = sz.center();

                    let w = cx[width];
                    let v = value.get(cx);
                    let r = SLIDER_THUMB_RADIUS;
                    let start_x = r;
                    let end_x = w - r;
                    let x = (1.0 - v) * start_x + v * (end_x);

                    let paint = vger
                        .state()
                        .create_fast_paint(Paint::Color(BUTTON_BACKGROUND_COLOR))
                        .unwrap();
                    vger.draw_shape(
                        &kurbo::Rect::new(
                            start_x as _,
                            c.y - SLIDER_WIDTH / 2.0 as _,
                            sz.size.width - 2.0 * r as _,
                            SLIDER_WIDTH as _,
                        ),
                        paint,
                    );
                    let paint = vger
                        .state()
                        .create_fast_paint(Paint::Color(AZURE_HIGHLIGHT_BACKGROUND))
                        .unwrap();
                    vger.draw_shape(
                        &kurbo::Rect::new(
                            start_x as _,
                            c.y - SLIDER_WIDTH / 2.0 as _,
                            x as _,
                            SLIDER_WIDTH as _,
                        ),
                        paint,
                    );
                    let paint = vger
                        .state()
                        .create_fast_paint(Paint::Color(opts.thumb))
                        .unwrap();
                    vger.draw_shape(&kurbo::Circle::new((x as f64, c.y as f64), r as f64), paint);
                })
                .geom(move |cx, sz, _| {
                    if sz.width != cx[width] {
                        cx[width] = sz.width;
                    }
                })
                .drag_s(value, move |v, delta, _, _| {
                    *v = (*v + delta.x / w).clamp(0.0, 1.0)
                })
            },
        )
        .role(accesskit::Role::Slider)
    })
}

impl<F> SliderMods for ModView<SliderOptions, F>
where
    ModView<SliderOptions, F>: View,
{
    fn thumb_color(self, color: Color) -> Self {
        let mut opts = self.value;
        opts.thumb = color;
        ModView {
            func: self.func,
            value: opts,
        }
    }
}

/// Vertical slider built from other Views.
pub fn vslider(
    value: f64,
    set_value: impl Fn(&mut Context, f64) + 'static + Copy,
) -> impl SliderMods {
    modview(move |opts: SliderOptions, _| {
        state(
            || 0.0,
            move |height, _| {
                canvas(move |cx, sz, vger| {
                    let h = cx[height];
                    let y = value * h;
                    let c = sz.center();
                    let paint = vger.color_paint(BUTTON_BACKGROUND_COLOR);
                    vger.fill_rect(
                        euclid::rect(c.x - SLIDER_WIDTH / 2.0, 0.0, SLIDER_WIDTH, sz.height()),
                        0.0,
                        paint,
                    );
                    let paint = vger.color_paint(opts.thumb);
                    vger.fill_circle([c.x, y], SLIDER_THUMB_RADIUS, paint);
                })
                .geom(move |cx, sz, _| {
                    if sz.height != cx[height] {
                        cx[height] = sz.height;
                    }
                })
                .drag(move |cx, delta, _, _| {
                    (set_value)(cx, (value + delta.y / cx[height]).clamp(0.0, 1.0));
                })
            },
        )
    })
}
