use rui::*;

fn main() {
    list(vec![7, 42], |i| {
        if *i == 7 {
            any_view(circle().build())
        } else {
            any_view(rectangle().build())
        }
        .padding(Auto)
    })
    .run()
}
