use fluss::{
    backend::{SkiaBackend, SkiaEnv},
    ApplicationState,
};
use winit::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder};

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let winit_window_builder = WindowBuilder::new()
        .with_title("Fluss")
        .with_inner_size(LogicalSize::new(800, 800))
        .with_transparent(true)
        .with_blur(true);

    let mut env = SkiaEnv::new(winit_window_builder, &event_loop);
    let mut app = ApplicationState::init(&mut env);

    // let font_mgr = FontMgr::new();
    // let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" height = "100" width = "100">
    //     <path d="M30,1h40l29,29v40l-29,29h-40l-29-29v-40z" stroke="#;000" fill="none"/>
    //     <path d="M31,3h38l28,28v38l-28,28h-38l-28-28v-38z" fill="#a23"/>
    //     <text x="50" y="68" font-size="48" fill="#FFF" text-anchor="middle"><![CDATA[410]]></text>
    //     </svg>"##;
    // let dom = SvgDom::from_str(svg, font_mgr).unwrap();

    event_loop
        .run(move |event, window_target| {
            app.handle_event(event, window_target, &mut env);
        })
        .expect("run() failed");
}
