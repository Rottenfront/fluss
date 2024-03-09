use defui::*;

fn main() {
    Application::new("Defui", |ctx, _| {
        // let first = (
        //     ctx.new_id(),
        //     Box::new(Filler::new(|| Color::new(1.0, 0.0, 0.0, 1.0))) as Box<dyn View>,
        // );
        // let second = (
        //     ctx.new_id(),
        //     Box::new(Filler::new(|| Color::TRANSPARENT)) as Box<dyn View>,
        // );
        // let third = (
        //     ctx.new_id(),
        //     Box::new(Filler::new(|| Color::new(0.0, 0.0, 1.0, 1.0))) as Box<dyn View>,
        // );
        // Stack::vstack(vec![first, second, third])
        // Stack::vstack(vec![first])
        Filler::new(|| Color::new(1.0, 0.0, 0.0, 1.0))
    })
    .run();
}
