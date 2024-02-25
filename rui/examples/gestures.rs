use rui::*;

fn anim_to(current: &mut Vec2, target: Vec2) -> bool {
    if *current != target {
        if (*current - target).length() < 0.01 {
            *current = target;
        } else {
            *current = current.lerp(target, 0.05);
        }
        true
    } else {
        false
    }
}

fn main() {
    hstack((
        circle()
            .color(RED_HIGHLIGHT.alpha(0.8))
            .build()
            .tap(|_cx| println!("tapped circle"))
            .padding(Auto),
        state(
            || Vec2::ZERO,
            move |off, _| {
                // target offset
                state(
                    || Vec2::ZERO,
                    move |anim_off, cx| {
                        // animated offset
                        rectangle()
                            .corner_radius(5.0)
                            .color(AZURE_HIGHLIGHT.alpha(0.8))
                            .build()
                            .offset(cx[anim_off])
                            .drag(move |cx, delta, state, _| {
                                cx[off] += delta;
                                cx[anim_off] = cx[off];
                                if state == GestureState::Ended {
                                    cx[off] = Vec2::ZERO;
                                }
                            })
                            .anim(move |cx, _dt| {
                                let mut v = cx[anim_off];
                                if anim_to(&mut v, cx[off]) {
                                    cx[anim_off] = v;
                                }
                            })
                            .padding(Auto)
                    },
                )
            },
        ),
    ))
    .run()
}
