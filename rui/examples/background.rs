use rui::*;

fn main() {
    text("this is a test")
        .padding(Auto)
        .background(
            rectangle()
                .corner_radius(5.0)
                .color(AZURE_HIGHLIGHT)
                .build(),
        )
        .run()
}
