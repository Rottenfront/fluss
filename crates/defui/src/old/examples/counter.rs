use defui::*;

fn main() {
    state(
        || 1,
        |count, cx| {
            vstack((
                cx[count].padding(Auto),
                button(text("increment"), move |cx| {
                    cx[count] += 1;
                })
                .padding(Auto),
            ))
        },
    )
    .run()
}
