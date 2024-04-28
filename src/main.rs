use fluss::*;

fn main() {
    run(
        hstack! {
            zstack! {
                Filler::new(|| Color::from_rgb8(255, 0, 0))
                // TextView::new(
                //     || "AAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA".into(),
                //     bind(Color::BLACK),
                //     bind(Font::MONOSPACE),
                // )
            },
            zstack! {
                Filler::new(|| Color::from_rgb8(0, 255, 0))
                // TextView::new(
                //     || "AAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA".into(),
                //     bind(Color::BLACK),
                //     bind(Font::MONOSPACE),
                // )
            },
            zstack! {
                Clickable::new(Filler::new(|| Color::from_rgb8(0, 0, 255)), |_, _, _| println!("press"))
                // TextView::new(
                //     || "AAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA".into(),
                //     bind(Color::BLACK),
                //     bind(Font::MONOSPACE),
                // )
            }
        },
        WindowProperties {
            title: "todo".into(),
            background_color: Color::BLACK,
            transparent: false,
            size: (1200, 800),
        },
    )
}
