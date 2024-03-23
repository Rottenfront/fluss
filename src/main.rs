use fluss::defui::shell::piet::Color;
use fluss::defui::*;
use shell::MouseButton;

fn main() {
    run(
        |ctx| {
            let first = {
                let first = ctx.push_view(Filler::new(|| Color::RED));
                let txt = ctx.push_view(TextView::new(
                    || "AAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA".into(),
                    bind(Color::BLACK),
                    bind(Font::MONOSPACE),
                ));
                ctx.push_view(Stack::zstack(vec![first, txt]))
            };
            let second = {
                let first = ctx.push_view(Filler::new(|| Color::GREEN));
                let txt = ctx.push_view(TextView::new(
                    || "AAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA".into(),
                    bind(Color::BLACK),
                    bind(Font::MONOSPACE),
                ));
                let button = ctx.push_view(Clickable::new(txt, |_, button| match button {
                    MouseButton::Left => println!("left"),
                    MouseButton::Right => println!("right"),
                    _ => println!("wtf"),
                }));
                ctx.push_view(Stack::zstack(vec![first, button]))
            };
            let third = {
                let first = ctx.push_view(Filler::new(|| Color::BLUE));
                let txt = ctx.push_view(TextView::new(
                    || "AAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA".into(),
                    bind(Color::BLACK),
                    bind(Font::MONOSPACE),
                ));
                ctx.push_view(Stack::zstack(vec![first, txt]))
            };

            Stack::hstack(vec![first, second, third])
        },
        WindowProperties {
            title: "wtf".into(),
            backdround_color: Color::GRAY,
        },
    )
}
