use fluss::defui::shell::piet::Color;
use fluss::{defui::*, hstack, zstack};

fn main() {
    run(
        |ctx| {
            hstack![
                ctx;
                zstack![
                    ctx;
                    Filler::new(|| Color::RED),
                    TextView::new(
                        || "AAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA".into(),
                        bind(Color::BLACK),
                        bind(Font::MONOSPACE),
                    )
                ],
                zstack![
                    ctx;
                    Filler::new(|| Color::GREEN),
                    TextView::new(
                        || "AAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA".into(),
                        bind(Color::BLACK),
                        bind(Font::MONOSPACE),
                    )
                ],
                zstack![
                    ctx;
                    Filler::new(|| Color::BLUE),
                    TextView::new(
                        || "AAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA".into(),
                        bind(Color::BLACK),
                        bind(Font::MONOSPACE),
                    )
                ]
            ]
        },
        WindowProperties {
            title: "wtf".into(),
            backdround_color: Color::GRAY,
        },
    )
}
