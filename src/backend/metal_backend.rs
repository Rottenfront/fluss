use cocoa::{
    appkit::NSView,
    base::id as cocoa_id,
};
use core_graphics_types::geometry::CGSize;
use foreign_types_shared::{ForeignType, ForeignTypeRef};
use metal::{Device, MTLPixelFormat, MetalLayer, CommandQueue};
use objc::
    runtime::YES
;
use raw_window_handle::HasWindowHandle;
use skia_safe::{gpu::{self, mtl, BackendRenderTarget, DirectContext, SurfaceOrigin}, Canvas, scalar, Size, ColorType};
use winit::{
    window::{WindowBuilder, Window},
    event_loop::EventLoop, dpi::PhysicalSize
};

pub struct SkiaEnv {
    window: Window,
    context: DirectContext,
    metal_layer: MetalLayer,
    command_queue: CommandQueue,
}

impl super::SkiaBackend for SkiaEnv {
    fn new<T>(winit_window_builder: WindowBuilder, event_loop: &EventLoop<T>) -> Self {
        let window = winit_window_builder.build(&event_loop).unwrap();

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

        let context = DirectContext::new_metal(&backend, None).unwrap();

        Self {
            window,
            metal_layer,
            context,
            command_queue,
        }
    }

    fn on_resize(&mut self, size: PhysicalSize<u32>) {
        self.metal_layer.set_drawable_size(CGSize::new(size.width as f64, size.height as f64));
    }

    fn request_redraw(&mut self) {
        self.window.request_redraw();
    }

    fn draw<F: FnOnce(&Canvas)>(&mut self, draw_func: F) {
        if let Some(drawable) = self.metal_layer.next_drawable() {
            let drawable_size = {
                let size = self.metal_layer.drawable_size();
                Size::new(size.width as scalar, size.height as scalar)
            };

            let mut surface = unsafe {
                let texture_info =
                    mtl::TextureInfo::new(drawable.texture().as_ptr() as mtl::Handle);

                let backend_render_target = BackendRenderTarget::new_metal(
                    (drawable_size.width as i32, drawable_size.height as i32),
                    &texture_info,
                );

                gpu::surfaces::wrap_backend_render_target(
                    &mut self.context,
                    &backend_render_target,
                    SurfaceOrigin::TopLeft,
                    ColorType::BGRA8888,
                    None,
                    None,
                )
                .unwrap()
            };

            draw_func(surface.canvas());

            self.context.flush_and_submit();
            drop(surface);

            let command_buffer = self.command_queue.new_command_buffer();
            command_buffer.present_drawable(drawable);
            command_buffer.commit();
        }
    }

    fn window(&mut self) -> &mut Window {
        &mut self.window
    }
}
