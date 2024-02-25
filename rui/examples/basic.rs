use rui::*;

fn main() {
    vstack((
        text("This is a test."),
        rectangle().build().flex(),
        text("This is another test."),
        rectangle().build().flex(),
    ))
    .run()
}
