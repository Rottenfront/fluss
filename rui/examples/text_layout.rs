use rui::*;

fn main() {
    canvas(|_cx, rect, vger| {

        let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

        let paint = vger.color_paint(MAGENTA.alpha(0.2));

        let font_size = 24;
        let break_width = Some(rect.width());

        let bounds = vger.state().text_bounds(lorem,break_width, FALLBACK_SERIF_FONT, font_size);

        // vger.stroke_rect(
        //     bounds.origin,
        //     bounds.max(),
        //     10.0,
        //     4.0,
        //     paint,
        // );

        let rects = vger.glyph_positions(lorem, font_size, break_width);

        let glyph_rect_paint = vger.color_paint(MAGENTA.alpha(0.1)).unwrap();

        for rect in rects {
            vger.draw_rect(rect, glyph_rect_paint);
        }

        let lines = vger.line_metrics(lorem, font_size, break_width);

        let line_rect_paint = vger.color_paint(RED_HIGHLIGHT.alpha(0.1)).unwrap();

        for line in lines {
            vger.draw_rect(line.bounds, line_rect_paint);
        }

        vger.draw_text(lorem, (0.0, 0.0).into(), break_width, font_size, FALLBACK_SERIF_FONT, vger.color_paint(TEXT_COLOR).unwrap());

    }).padding(Auto).run()
}
