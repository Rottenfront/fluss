#[cfg(feature = "metal-render")]
fn main() {
    use cocoa::{
        appkit::{NSView, NSWindow},
        base::id as cocoa_id,
    };
    use core_graphics_types::geometry::CGSize;
    use foreign_types_shared::{ForeignType, ForeignTypeRef};
    use metal::{Device, MTLPixelFormat, MetalLayer};
    use objc::{
        rc::autoreleasepool,
        runtime::{NO, YES},
    };
    use skia_safe::gpu::{self, mtl, BackendRenderTarget, DirectContext, SurfaceOrigin};
    use winit::{
        platform::macos::{WindowBuilderExtMacOS, WindowExtMacOS},
        raw_window_handle::HasWindowHandle,
    };
    let app = ApplicationState {
        monospace_font: Font::new(
            FontMgr::new()
                .match_family_style(
                    "Cascadia Code PL",
                    FontStyle::new(Weight::NORMAL, Width::NORMAL, Slant::Upright),
                )
                .unwrap(),
            14.0,
        ),
    };

    let size = LogicalSize::new(800, 600);

    let events_loop = EventLoop::new().expect("Failed to create event loop");

    let window = WindowBuilder::new()
        .with_inner_size(size)
        .with_transparent(true)
        .with_titlebar_transparent(true)
        .with_title_hidden(true)
        .with_fullsize_content_view(true)
        .with_title("Skia Metal Winit Example".to_string())
        .build(&events_loop)
        .unwrap();

    let window_handle = window
        .window_handle()
        .expect("Failed to retrieve a window handle");

    let raw_window_handle = window_handle.as_raw();

    let device = Device::system_default().expect("no device found");

    let metal_layer = {
        let draw_size = window.inner_size();
        let layer = MetalLayer::new();
        layer.set_device(&device);
        layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        layer.set_presents_with_transaction(false);
        // Disabling this option allows Skia's Blend Mode to work.
        // More about: https://developer.apple.com/documentation/quartzcore/cametallayer/1478168-framebufferonly
        layer.set_framebuffer_only(false);

        unsafe {
            let view = match raw_window_handle {
                raw_window_handle::RawWindowHandle::AppKit(appkit) => appkit.ns_view.as_ptr(),
                _ => panic!("Wrong window handle type"),
            } as cocoa_id;
            // view.setTitlebarAppearsTransparent_(NO);
            view.setWantsLayer(YES);
            view.setLayer(layer.as_ref() as *const _ as _);
        }
        layer.set_drawable_size(CGSize::new(draw_size.width as f64, draw_size.height as f64));
        layer
    };

    let command_queue = device.new_command_queue();

    let backend = unsafe {
        mtl::BackendContext::new(
            device.as_ptr() as mtl::Handle,
            command_queue.as_ptr() as mtl::Handle,
            std::ptr::null(),
        )
    };

    let mut context = DirectContext::new_metal(&backend, None).unwrap();

    events_loop
        .run(move |event, window_target| {
            autoreleasepool(|| match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => window_target.exit(),
                    WindowEvent::Resized(size) => {
                        metal_layer
                            .set_drawable_size(CGSize::new(size.width as f64, size.height as f64));
                        window.request_redraw()
                    }
                    WindowEvent::RedrawRequested => {
                        if let Some(drawable) = metal_layer.next_drawable() {
                            let drawable_size = {
                                let size = metal_layer.drawable_size();
                                Size::new(size.width as scalar, size.height as scalar)
                            };

                            let mut surface = unsafe {
                                let texture_info = mtl::TextureInfo::new(
                                    drawable.texture().as_ptr() as mtl::Handle,
                                );

                                let backend_render_target = BackendRenderTarget::new_metal(
                                    (drawable_size.width as i32, drawable_size.height as i32),
                                    &texture_info,
                                );

                                gpu::surfaces::wrap_backend_render_target(
                                    &mut context,
                                    &backend_render_target,
                                    SurfaceOrigin::TopLeft,
                                    ColorType::BGRA8888,
                                    None,
                                    None,
                                )
                                .unwrap()
                            };

                            app.draw(surface.canvas());

                            context.flush_and_submit();
                            drop(surface);

                            let command_buffer = command_queue.new_command_buffer();
                            command_buffer.present_drawable(drawable);
                            command_buffer.commit();
                        }
                    }
                    _ => (),
                },
                Event::LoopExiting => {}
                _ => {}
            });
        })
        .expect("run() failed");
}
