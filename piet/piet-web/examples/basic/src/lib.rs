// Copyright 2019 the Piet Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Basic example of rendering in the browser

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlCanvasElement};

use piet::RenderContext;
use piet_web::WebRenderContext;

//TODO: figure out how to dynamically select the sample?
const SAMPLE_PICTURE_NO: usize = 11;

// Copyright 2020 the Piet Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use piet::kurbo::{Circle, Line, Point, Size};
use piet::{Color, Error, Text, TextLayout, TextLayoutBuilder};

pub const SIZE: Size = Size::new(440., 400.);
pub const DOT_RADIUS: f64 = 2.0;
pub const TEST_ADVANCE: f64 = 23.4;

static TEXT_EN: &str = r#"Philosophers often behave like little children who scribble some marks on a piece of paper at random and then ask the grown-up "What's that?" — It happened like this: the grown-up had drawn pictures for the child several times and said "this is a man," "this is a house," etc. And then the child makes some marks too and asks: what's this then?"#;

static TEXT_AR: &str = r#"لكن لا بد أن أوضح لك أن كل هذه الأفكار المغلوطة حول استنكار  النشوة وتمجيد الألم نشأت بالفعل، وسأعرض لك التفاصيل لتكتشف حقيقة وأساس تلك السعادة البشرية، فلا أحد يرفض أو يكره أو يتجنب الشعور بالسعادة، ولكن بفضل هؤلاء الأشخاص الذين لا"#;

const LIGHT_GREY: Color = Color::grey8(0xc0);
const RED: Color = Color::rgb8(255, 0, 0);
const BLUE: Color = Color::rgb8(0, 0, 255);

pub fn draw<R: RenderContext>(rc: &mut R) -> Result<(), Error> {
    rc.clear(None, LIGHT_GREY);
    let layout_en_start = rc
        .text()
        .new_text_layout(TEXT_EN)
        .alignment(piet::TextAlignment::Start)
        .max_width(200.0)
        .build()?;
    let layout_en_center = rc
        .text()
        .new_text_layout(TEXT_EN)
        .alignment(piet::TextAlignment::Center)
        .max_width(200.0)
        .build()?;
    let layout_ar_start = rc
        .text()
        .new_text_layout(TEXT_AR)
        .alignment(piet::TextAlignment::Start)
        .max_width(200.0)
        .build()?;
    let layout_ar_just = rc
        .text()
        .new_text_layout(TEXT_AR)
        .alignment(piet::TextAlignment::Justified)
        .max_width(200.0)
        .build()?;
    let ar_y = ((SIZE.height - layout_ar_start.size().height) - 32.0).max(0.0);

    visualize_hit_testing(rc, layout_en_start, Point::new(16.0, 32.0))?;
    visualize_hit_testing(rc, layout_en_center, Point::new(232.0, 32.0))?;
    visualize_hit_testing(rc, layout_ar_start, Point::new(16.0, ar_y))?;
    visualize_hit_testing(rc, layout_ar_just, Point::new(232.0, ar_y))?;
    Ok(())
}

fn visualize_hit_testing<R: RenderContext>(
    rc: &mut R,
    layout: R::TextLayout,
    origin: Point,
) -> Result<(), Error> {
    let layout_rect = layout.size().to_rect() + origin.to_vec2();
    rc.fill(layout_rect, &Color::WHITE);

    let mut y = origin.y - 20.;
    while y < (layout_rect.max_y() + TEST_ADVANCE) {
        let mut x = origin.x - 8.0;
        while x - origin.x < layout.size().width + 8.0 {
            let point = Point::new(x, y);
            let test_point = layout.hit_test_point(point - origin.to_vec2());
            let test_pos = layout.hit_test_text_position(test_point.idx);
            let hit_point = test_pos.point + origin.to_vec2();

            let color = if test_point.is_inside { &RED } else { &BLUE };

            let line = Line::new(point, hit_point);
            let dot1 = Circle::new(point, DOT_RADIUS);
            let dot = Circle::new(hit_point, DOT_RADIUS);
            rc.stroke(dot1, color, 0.5);
            rc.stroke(line, color, 0.5);
            rc.fill(dot, color);
            x += TEST_ADVANCE;
        }
        y += TEST_ADVANCE;
    }
    rc.draw_text(&layout, origin);
    Ok(())
}

#[wasm_bindgen]
pub fn run() {
    #[cfg(feature = "console_error_panic_hook")]
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let window = window().unwrap();
    let canvas = window
        .document()
        .unwrap()
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let dpr = window.device_pixel_ratio();
    canvas.set_width((canvas.offset_width() as f64 * dpr) as u32);
    canvas.set_height((canvas.offset_height() as f64 * dpr) as u32);
    let _ = context.scale(dpr, dpr);

    let mut piet_context = WebRenderContext::new(context, window);

    draw(&mut piet_context).unwrap();
    piet_context.finish().unwrap();
}
