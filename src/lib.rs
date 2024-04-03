// 
// use fluss::{defui::*, hstack, vstack, zstack};
//
// fn main() {
//     run(
//         hstack! {
//             zstack!{
//                 Filler::new(|| Color::RED),
//                 TextView::new(
//                     || "AAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA".into(),
//                     bind(Color::BLACK),
//                     bind(Font::MONOSPACE),
//                 )
//             },
//             zstack!{
//                 Filler::new(|| Color::GREEN),
//                 TextView::new(
//                     || "AAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA".into(),
//                     bind(Color::BLACK),
//                     bind(Font::MONOSPACE),
//                 )
//             },
//             vstack!{
//                 Clickable::new(Filler::new(|| Color::BLUE), |_, _, _| println!("press")),
//                 TextView::new(
//                     || "AAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA".into(),
//                     bind(Color::BLACK),
//                     bind(Font::MONOSPACE),
//                 )
//             }
//         },
//         WindowProperties {
//             title: "title".into(),
//             background_color: Color::GRAY,
//         },
//     )
// }

use std::time::{Duration, Instant};

use winit::{
    event::{Event as WEvent, WindowEvent, Modifiers, KeyEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use iced_wgpu::core::Renderer as CoreRenderer;
use iced_wgpu::Backend;
use iced_wgpu::{
    core::{renderer::Quad, Border, Color, Rectangle, Size},
    graphics::{Primitive, Viewport},
    primitive::Custom,
    wgpu, Renderer, Settings,
};
use winit::dpi::PhysicalSize;

struct UIApp<V: View + 'static> {
    root_view: V,

    renderer: Renderer,
    width: u32,
    height: u32,
    scale: f64,
    window: Window,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    frame_duration: Duration,
    previous_frame_start: Instant,
    modifiers: Modifiers,
}

mod app;
mod context;
mod event;
mod filler;
mod modifiers;
mod stack;
mod text;
mod view;

pub use context::*;
pub use event::*;
pub use filler::*;
pub use modifiers::*;
pub use stack::*;
pub use text::*;
pub use view::*;

pub use flo_binding::{bind, Binding, Bound, MutableBound};

use std::any::Any;

pub struct WindowProperties {
    pub title: String,
    pub background_color: Color,
}

#[macro_export]
macro_rules! zstack {
    // The pattern for a single `eval`
    {$($view:expr),+} => {
        {
            let mut views = vec![];
            $(
                views.push(Box::new($view) as Box<(dyn View + 'static)>);
            )+
            Stack::zstack(views)
        }
    };
}

#[macro_export]
macro_rules! hstack {
    // The pattern for a single `eval`
    {$($view:expr),+} => {
        {
            let mut views = vec![];
            $(
                views.push(Box::new($view) as Box<(dyn View + 'static)>);
            )+
            Stack::hstack(views)
        }
    };
}

#[macro_export]
macro_rules! vstack {
    // The pattern for a single `eval`
    {$($view:expr),+} => {
        {
            let mut views = vec![];
            $(
                views.push(Box::new($view) as Box<(dyn View + 'static)>);
            )+
            Stack::vstack(views)
        }
    };
}

// pub fn run<V: View + 'static>(view: V, window_properties: WindowProperties) {
//     tracing_subscriber::fmt().init();
//     let app = Application::new().unwrap();
//     let mut builder = WindowBuilder::new(app.clone());

//     builder.set_title(&window_properties.title);
//     // we set transparent to true so user can make window background transparent
//     builder.set_transparent(true);

//     let uiapp = app::UIApp::new(view, window_properties);
//     builder.set_handler(Box::new(uiapp));

//     let window = builder.build().unwrap();

//     window.show();

//     app.run(None);
// }


#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    env_logger::init();

    let mut width = 1200;
    let mut height = 800;
    let mut scale = 1.0;

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_blur(true)
        .with_transparent(true)
        .with_inner_size(PhysicalSize::new(width, height))
        .build(&event_loop)
        .unwrap();

    let size = window.inner_size();

    // The instance is a handle to our GPU
    // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let surface = instance.create_surface(&window).unwrap();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
            },
            // Some(&std::path::Path::new("trace")), // Trace path
            None,
        )
        .await
        .unwrap();

    let surface_caps = surface.get_capabilities(&adapter);
    // Shader code in this tutorial assumes an Srgb surface texture. Using a different
    // one will result all the colors comming out darker. If you want to support non
    // Srgb surfaces, you'll need to account for that when drawing to the frame.
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);
    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        desired_maximum_frame_latency: 0,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &config);
    let settings = Settings::from_env();
    let font = settings.default_font.clone();
    let font_size = settings.default_text_size;

    let backend = Backend::new(
        &device,
        &queue,
        settings,
        wgpu::TextureFormat::Bgra8UnormSrgb,
    );

    let mut renderer = Renderer::new(backend, font, font_size);
    let expected_frame_length_seconds = 1.0 / 144.0;
    let frame_duration = Duration::from_secs_f32(expected_frame_length_seconds);

    let mut previous_frame_start = Instant::now();
    let mut modifiers = Modifiers::default();
    event_loop
        .run(|event, window_target| {
            let frame_start = Instant::now();
            let mut draw_frame = false;

            if let WEvent::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested => {
                        window_target.exit();
                        return;
                    }
                    WindowEvent::Resized(physical_size) => {
                        /* First resize the opengl drawable */
                        let (new_width, new_height): (u32, u32) = physical_size.into();

                        if new_width > 0 && new_height > 0 {
                            width = new_width;
                            height = new_height;
                            config.width = new_width;
                            config.height = new_height;
                            surface.configure(&device, &config);
                        }
                    }
                    WindowEvent::ModifiersChanged(new_modifiers) => modifiers = new_modifiers,
                    WindowEvent::KeyboardInput {
                        event: KeyEvent { logical_key, .. },
                        ..
                    } => {
                        if modifiers.state().super_key() && logical_key == "q" {
                            window_target.exit();
                        }
                        window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        draw_frame = true;
                    }
                    _ => (),
                }
            }
            let frame_time = frame_start - previous_frame_start;
            if frame_time > frame_duration {
                draw_frame = true;
                previous_frame_start = frame_start;
            }
            if draw_frame {
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle {
                            x: 0.0,
                            y: 0.0,
                            width: 100.0,
                            height: 100.0,
                        },
                        border: Border {
                            color: Color::new(1.0, 0.0, 0.0, 1.0),
                            radius: 10.0.into(),
                            width: 10.0,
                        },
                        ..Default::default()
                    },
                    Color::BLACK,
                );
                'draw: {
                    let output = match surface.get_current_texture() {
                        Ok(output) => output,
                        Err(_) => {
                            break 'draw;
                        }
                    };
                    let view = output
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    let mut encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Render Encoder"),
                        });

                    renderer.with_primitives(|backend, primitives| {
                        backend.present::<&str>(
                            &device,
                            &queue,
                            &mut encoder,
                            None,
                            wgpu::TextureFormat::Bgra8UnormSrgb,
                            &view,
                            primitives,
                            &Viewport::with_physical_size(Size::new(width, height), scale),
                            &[],
                        );
                        &[] as &[Primitive<Custom>]
                    });
                    queue.submit(std::iter::once(encoder.finish()));
                    output.present();
                }

            }

            window_target.set_control_flow(ControlFlow::WaitUntil(
                previous_frame_start + frame_duration,
            ))
        })
        .expect("Failed to run event loop");
}