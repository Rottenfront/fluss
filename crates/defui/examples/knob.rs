use defui::*;

#[derive(Debug, Default)]
struct MyState {
    x: f64,
}

fn main() {
    state(MyState::default, |state, cx| {
        vstack((
            format!("value: {:?}", cx[state]).padding(Auto),
            map(
                cx[state].x * 0.01,
                move |v, cx| cx[state].x = v * 100.0,
                |s, _| knob(s).padding(Auto),
            ),
        ))
    })
    .run()
}