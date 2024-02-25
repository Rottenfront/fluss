use std::time::{Duration, Instant};
use trist::*;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[cfg(feature = "skia")]
fn main() {
    static EXPECTED_FRAME_DURATION: f32 = 1.0 / 60.0;
    let el = EventLoop::new().expect("Failed to create event loop");
    let winit_window_builder = WindowBuilder::new()
        .with_title("Fluss")
        .with_inner_size(LogicalSize::new(800, 800))
        .with_transparent(true)
        .with_blur(true);

    let mut env = DrawerEnv::new(winit_window_builder, &el);

    // let font_mgr = FontMgr::new();
    // let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" height = "100" width = "100">
    //     <path d="M30,1h40l29,29v40l-29,29h-40l-29-29v-40z" stroke="#;000" fill="none"/>
    //     <path d="M31,3h38l28,28v38l-28,28h-38l-28-28v-38z" fill="#a23"/>
    //     <text x="50" y="68" font-size="48" fill="#FFF" text-anchor="middle"><![CDATA[410]]></text>
    //     </svg>"##;
    // let dom = SvgDom::from_str(svg, font_mgr).unwrap();

    let mut previous_frame_start = Instant::now();
    let frame_duration = Duration::from_secs_f32(EXPECTED_FRAME_DURATION);

    let state = env.get_drawer_state();
    let font = state
        .create_font(Font {
            name: "CaskaydiaCove Nerd Font".to_string(),
            size: 13.0,
            weight: Weight::Normal,
            width: Width::Normal,
        })
        .unwrap();

    el.run(move |event, window_target| {
        let frame_start = Instant::now();
        let mut draw_frame = false;
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::CloseRequested => {
                    window_target.exit();
                    return;
                }
                WindowEvent::Resized(physical_size) => {
                    env.on_resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    draw_frame = true;
                }
                _ => (),
            }
        }
        if frame_start - previous_frame_start > frame_duration {
            draw_frame = true;
            previous_frame_start = frame_start;
        }

        if draw_frame {
            let (canvas, state) = env.get_drawer();
            let mut drawer = Drawer::new(canvas, state);
            drawer.clear(Color {
                a: 0.1,
                r: 0.0,
                b: 0.0,
                g: 0.0,
            });
            let black = drawer
                .state()
                .create_fast_paint(Paint::Color(Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }))
                .unwrap();
            let white = drawer
                .state()
                .create_fast_paint(Paint::Color(Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                }))
                .unwrap();
            drawer.draw_shape(&gcl::Circle::new((100.0, 100.0), 100.0), black);
            drawer.draw_text("chlen", (100.0, 100.0).into(), None, 13.0, font, white);
            drop(drawer);
            env.draw();
        }

        window_target.set_control_flow(ControlFlow::WaitUntil(
            previous_frame_start + frame_duration,
        ))
    })
    .expect("run() failed");
}
