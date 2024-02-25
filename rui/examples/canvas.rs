use rui::*;

fn main() {
    vstack((
        canvas(|_, rect, vger| {
            // let paint = vger.linear_gradient(
            //     [-100.0, -100.0],
            //     [100.0, 100.0],
            //     AZURE_HIGHLIGHT,
            //     RED_HIGHLIGHT,
            //     0.0,
            // );
            let paint = vger.color_paint(RED_HIGHLIGHT).unwrap();

            let radius = 100.0;
            vger.draw_circle(rect.center(), radius, paint);
        }),
        canvas(|_, rect, vger| {
            // let paint = vger.linear_gradient(
            //     [-100.0, -100.0],
            //     [100.0, 100.0],
            //     AZURE_HIGHLIGHT,
            //     RED_HIGHLIGHT,
            //     0.0,
            // );
            let paint = vger.color_paint(RED_HIGHLIGHT).unwrap();

            let radius = 100.0;
            vger.draw_circle(rect.center(), radius, paint);
        }),
        circle().color(RED_HIGHLIGHT).build().padding(Auto),
        rectangle()
            .corner_radius(5.0)
            .color(AZURE_HIGHLIGHT)
            .build()
            .padding(Auto),
    ))
    .run()
}
