use fluss::*;

fn main() {
    run(
        hstack! {
            Filler::new(|| Color::new(1.0, 0.0, 0.0, 1.0)),
            Filler::new(|| Color::new(0.0, 1.0, 0.0, 1.0)),
            Filler::new(|| Color::new(0.0, 0.0, 1.0, 1.0))
        },
        WindowProperties {
            title: "todo".into(),
            background_color: Color::BLACK,
            transparent: false,
            size: (1200, 800),
        },
    )
}

// use fluss::{Color, RectQuad, Renderer};
// use winit::{
//     event::{Event, WindowEvent},
//     event_loop::{ControlFlow, EventLoop},
//     keyboard::ModifiersState,
// };

// use std::time::{Duration, Instant};

// pub fn main() -> Result<(), Box<dyn std::error::Error>> {
//     tracing_subscriber::fmt::init();

//     // Initialize winit
//     let event_loop = EventLoop::new()?;

//     let window = winit::window::Window::new(&event_loop)?;

//     let mut resized = false;

//     // Initialize scene and GUI controls
//     let mut renderer = Renderer::new(&window);

//     // Run event loop
//     event_loop.run(move |event, window_target| {
//         match event {
//             Event::WindowEvent {
//                 event: WindowEvent::RedrawRequested,
//                 ..
//             } => {
//                 if resized {
//                     renderer.resize(window.inner_size(), window.scale_factor());
//                     resized = false;
//                 }
//                 renderer.fill_rect(
//                     RectQuad::new((0.0, 0.0).into(), (100.0, 100.0).into())
//                         .with_color(Color::BLACK)
//                         .with_radius(10.0)
//                         .with_border_width(10.0)
//                         .with_border_color(Color::new(1.0, 0.0, 0.0, 1.0)),
//                 );

//                 renderer.present();
//             }
//             Event::WindowEvent { event, .. } => match event {
//                 WindowEvent::CursorMoved { position, .. } => {
//                     // cursor_position = Some(position);
//                     window.request_redraw();
//                 }
//                 WindowEvent::ModifiersChanged(new_modifiers) => {
//                     // modifiers = new_modifiers.state();
//                 }
//                 WindowEvent::Resized(_) => {
//                     resized = true;
//                     window.request_redraw();
//                 }
//                 WindowEvent::CloseRequested => {
//                     window_target.exit();
//                 }
//                 _ => {}
//             },
//             _ => {}
//         }
//     })?;

//     Ok(())
// }
