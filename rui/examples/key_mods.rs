use rui::*;

fn main() {
    hstack((
        zstack((
            circle()
                .color(RED_HIGHLIGHT.alpha(0.8))
                .build()
                .tap(|cx| println!("tapped circle, key modifiers state: {:?}", cx.key_mods))
                .padding(Auto),
            text("Tap (inside circle)"),
        )),
        zstack((
            rectangle()
                .corner_radius(5.0)
                .color(AZURE_HIGHLIGHT_BACKGROUND)
                .build()
                .drag(|cx, delta, _state, _mouse_button| {
                    println!(
                        "dragging: {:?}, key modifiers state: {:?}",
                        delta, cx.key_mods
                    )
                })
                .padding(Auto),
            text("Drag (inside rectangle)").padding(Auto),
        )),
        text("Handle key pressed")
            .key(|cx, key| println!("key: {:?}, key modifiers state: {:?}", key, cx.key_mods))
            .padding(Auto),
    ))
    .run()
}
