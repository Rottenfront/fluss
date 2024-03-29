use fluss::defui::shell::piet::Color;
use fluss::{defui::*, hstack, vstack, zstack};

fn main() {
    run(
        hstack! {
            zstack!{
                Filler::new(|| Color::RED),
                TextView::new(
                    || "AAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA".into(),
                    bind(Color::BLACK),
                    bind(Font::MONOSPACE),
                )
            },
            zstack!{
                Filler::new(|| Color::GREEN),
                TextView::new(
                    || "AAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA".into(),
                    bind(Color::BLACK),
                    bind(Font::MONOSPACE),
                )
            },
            vstack!{
                Clickable::new(Filler::new(|| Color::BLUE), |_, _, _| println!("press")),
                TextView::new(
                    || "AAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA".into(),
                    bind(Color::BLACK),
                    bind(Font::MONOSPACE),
                )
            }

        },
        WindowProperties {
            title: "wtf".into(),
            backdround_color: Color::GRAY,
        },
    )
}
