use skia_safe::{scalar, Canvas, Color4f, ColorType, Paint, Point, Rect, Size};

#[cfg(target_os = "macos")]
fn main() {
    use cocoa::{appkit::NSView, base::id as cocoa_id};
    use core_graphics_types::geometry::CGSize;
    use foreign_types_shared::{ForeignType, ForeignTypeRef};
    use metal::{Device, MTLPixelFormat, MetalLayer};
    use objc::{rc::autoreleasepool, runtime::YES};
    use skia_safe::gpu::{self, mtl, BackendRenderTarget, DirectContext, SurfaceOrigin};
    use winit::{
        dpi::LogicalSize,
        event::{Event, WindowEvent},
        event_loop::EventLoop,
        raw_window_handle::HasWindowHandle,
        window::WindowBuilder,
    };

    let size = LogicalSize::new(800, 600);

    let events_loop = EventLoop::new().expect("Failed to create event loop");

    let window = WindowBuilder::new()
        .with_inner_size(size)
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

                            draw(surface.canvas());

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

/// Renders a rectangle that occupies exactly half of the canvas
fn draw(canvas: &Canvas) {
    let canvas_size = Size::from(canvas.base_layer_size());

    canvas.clear(Color4f::new(1.0, 1.0, 1.0, 1.0));

    let rect_size = canvas_size / 2.0;
    let rect = Rect::from_point_and_size(
        Point::new(
            (canvas_size.width - rect_size.width) / 2.0,
            (canvas_size.height - rect_size.height) / 2.0,
        ),
        rect_size,
    );
    canvas.draw_rect(rect, &Paint::new(Color4f::new(0.0, 0.0, 1.0, 1.0), None));
}

#[cfg(feature = "gl")]
fn main() {
    use std::{
        ffi::CString,
        num::NonZeroU32,
        time::{Duration, Instant},
    };

    use gl::types::*;
    use gl_rs as gl;
    use glutin::{
        config::{ConfigTemplateBuilder, GlConfig},
        context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext},
        display::{GetGlDisplay, GlDisplay},
        prelude::{GlSurface, NotCurrentGlContext},
        surface::{Surface as GlutinSurface, SurfaceAttributesBuilder, WindowSurface},
    };
    use glutin_winit::DisplayBuilder;
    use raw_window_handle::HasRawWindowHandle;
    use winit::{
        dpi::LogicalSize,
        event::{Event, KeyEvent, Modifiers, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::{Window, WindowBuilder},
    };

    use skia_safe::{
        gpu::{self, backend_render_targets, gl::FramebufferInfo, SurfaceOrigin},
        Color, ColorType, Surface,
    };

    let el = EventLoop::new().expect("Failed to create event loop");
    let winit_window_builder = WindowBuilder::new()
        .with_title("rust-skia-gl-window")
        .with_inner_size(LogicalSize::new(800, 800));

    let template = ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_transparency(true);

    let display_builder = DisplayBuilder::new().with_window_builder(Some(winit_window_builder));
    let (window, gl_config) = display_builder
        .build(&el, template, |configs| {
            // Find the config with the minimum number of samples. Usually Skia takes care of
            // anti-aliasing and may not be able to create appropriate Surfaces for samples > 0.
            // See https://github.com/rust-skia/rust-skia/issues/782
            // And https://github.com/rust-skia/rust-skia/issues/764
            configs
                .reduce(|accum, config| {
                    let transparency_check = config.supports_transparency().unwrap_or(false)
                        & !accum.supports_transparency().unwrap_or(false);

                    if transparency_check || config.num_samples() < accum.num_samples() {
                        config
                    } else {
                        accum
                    }
                })
                .unwrap()
        })
        .unwrap();
    println!("Picked a config with {} samples", gl_config.num_samples());
    let window = window.expect("Could not create window with OpenGL context");
    let raw_window_handle = window.raw_window_handle();

    // The context creation part. It can be created before surface and that's how
    // it's expected in multithreaded + multiwindow operation mode, since you
    // can send NotCurrentContext, but not Surface.
    let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));

    // Since glutin by default tries to create OpenGL core context, which may not be
    // present we should try gles.
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(Some(raw_window_handle));
    let not_current_gl_context = unsafe {
        gl_config
            .display()
            .create_context(&gl_config, &context_attributes)
            .unwrap_or_else(|_| {
                gl_config
                    .display()
                    .create_context(&gl_config, &fallback_context_attributes)
                    .expect("failed to create context")
            })
    };

    let (width, height): (u32, u32) = window.inner_size().into();

    let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        raw_window_handle,
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
    );

    let gl_surface = unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &attrs)
            .expect("Could not create gl window surface")
    };

    let gl_context = not_current_gl_context
        .make_current(&gl_surface)
        .expect("Could not make GL context current when setting up skia renderer");

    gl::load_with(|s| {
        gl_config
            .display()
            .get_proc_address(CString::new(s).unwrap().as_c_str())
    });
    let interface = skia_safe::gpu::gl::Interface::new_load_with(|name| {
        if name == "eglGetCurrentDisplay" {
            return std::ptr::null();
        }
        gl_config
            .display()
            .get_proc_address(CString::new(name).unwrap().as_c_str())
    })
    .expect("Could not create interface");

    let mut gr_context = skia_safe::gpu::DirectContext::new_gl(Some(interface), None)
        .expect("Could not create direct context");

    let fb_info = {
        let mut fboid: GLint = 0;
        unsafe { gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid) };

        FramebufferInfo {
            fboid: fboid.try_into().unwrap(),
            format: skia_safe::gpu::gl::Format::RGBA8.into(),
            ..Default::default()
        }
    };

    fn create_surface(
        window: &Window,
        fb_info: FramebufferInfo,
        gr_context: &mut skia_safe::gpu::DirectContext,
        num_samples: usize,
        stencil_size: usize,
    ) -> Surface {
        let size = window.inner_size();
        let size = (
            size.width.try_into().expect("Could not convert width"),
            size.height.try_into().expect("Could not convert height"),
        );
        let backend_render_target =
            backend_render_targets::make_gl(size, num_samples, stencil_size, fb_info);

        gpu::surfaces::wrap_backend_render_target(
            gr_context,
            &backend_render_target,
            SurfaceOrigin::BottomLeft,
            ColorType::RGBA8888,
            None,
            None,
        )
        .expect("Could not create skia surface")
    }
    let num_samples = gl_config.num_samples() as usize;
    let stencil_size = gl_config.stencil_size() as usize;

    let surface = create_surface(&window, fb_info, &mut gr_context, num_samples, stencil_size);

    let mut frame = 0usize;

    // Guarantee the drop order inside the FnMut closure. `Window` _must_ be dropped after
    // `DirectContext`.
    //
    // https://github.com/rust-skia/rust-skia/issues/476
    struct Env {
        surface: Surface,
        gl_surface: GlutinSurface<WindowSurface>,
        gr_context: skia_safe::gpu::DirectContext,
        gl_context: PossiblyCurrentContext,
        window: Window,
    }

    let mut env = Env {
        surface,
        gl_surface,
        gl_context,
        gr_context,
        window,
    };
    let mut previous_frame_start = Instant::now();
    let mut modifiers = Modifiers::default();
    let expected_frame_length_seconds = 1.0 / 60.0;
    let frame_duration = Duration::from_secs_f32(expected_frame_length_seconds);

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
                    env.surface = create_surface(
                        &env.window,
                        fb_info,
                        &mut env.gr_context,
                        num_samples,
                        stencil_size,
                    );
                    /* First resize the opengl drawable */
                    let (width, height): (u32, u32) = physical_size.into();

                    env.gl_surface.resize(
                        &env.gl_context,
                        NonZeroU32::new(width.max(1)).unwrap(),
                        NonZeroU32::new(height.max(1)).unwrap(),
                    );
                }
                WindowEvent::ModifiersChanged(new_modifiers) => modifiers = new_modifiers,
                WindowEvent::KeyboardInput {
                    event: KeyEvent { logical_key, .. },
                    ..
                } => {
                    if modifiers.state().super_key() && logical_key == "q" {
                        window_target.exit();
                    }
                    frame = frame.saturating_sub(10);
                    env.window.request_redraw();
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
            frame += 1;
            let canvas = env.surface.canvas();
            draw(&canvas);
            renderer::render_frame(frame % 360, 12, 60, canvas);
            env.gr_context.flush_and_submit();
            env.gl_surface.swap_buffers(&env.gl_context).unwrap();
        }

        window_target.set_control_flow(ControlFlow::WaitUntil(
            previous_frame_start + frame_duration,
        ))
    })
    .expect("run() failed");
}

#[cfg(not(all(target_os = "windows", feature = "d3d")))]
fn main() {
    println!("This example requires the `d3d` feature to be enabled on Windows.");
    println!("Run it with `cargo run --example d3d-window --features d3d`");
}

#[cfg(all(target_os = "windows", feature = "d3d"))]
fn main() -> anyhow::Result<()> {
    // NOTE: Most of code is from https://github.com/microsoft/windows-rs/blob/02db74cf5c4796d970e6d972cdc7bc3967380079/crates/samples/windows/direct3d12/src/main.rs

    use std::ptr;

    use anyhow::Result;
    use skia_safe::{
        gpu::{
            d3d::{BackendContext, TextureResourceInfo},
            surfaces, BackendRenderTarget, DirectContext, Protected, SurfaceOrigin,
        },
        paint, Color, ColorType, Paint, Rect,
    };
    use windows::{
        core::ComInterface,
        Win32::{
            Foundation::HWND,
            Graphics::{
                Direct3D::D3D_FEATURE_LEVEL_11_0,
                Direct3D12::{
                    D3D12CreateDevice, ID3D12CommandQueue, ID3D12DescriptorHeap, ID3D12Device,
                    ID3D12Resource, D3D12_COMMAND_LIST_TYPE_DIRECT, D3D12_COMMAND_QUEUE_DESC,
                    D3D12_COMMAND_QUEUE_FLAG_NONE, D3D12_CPU_DESCRIPTOR_HANDLE,
                    D3D12_DESCRIPTOR_HEAP_DESC, D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                    D3D12_RESOURCE_STATE_COMMON,
                },
                Dxgi::{
                    Common::{
                        DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC,
                        DXGI_STANDARD_MULTISAMPLE_QUALITY_PATTERN,
                    },
                    CreateDXGIFactory1, IDXGIAdapter1, IDXGIFactory4, IDXGISwapChain3,
                    DXGI_ADAPTER_FLAG, DXGI_ADAPTER_FLAG_NONE, DXGI_ADAPTER_FLAG_SOFTWARE,
                    DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_EFFECT_FLIP_DISCARD,
                    DXGI_USAGE_RENDER_TARGET_OUTPUT,
                },
            },
        },
    };
    use winit::{
        event::{Event, WindowEvent},
        keyboard::{Key, NamedKey},
    };

    let event_loop = winit::event_loop::EventLoop::new()?;
    let winit_window_builder = winit::window::WindowBuilder::new()
        .with_title("rust-skia-gl-window")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 800));

    let window = winit_window_builder.build(&event_loop)?;

    const FRAME_COUNT: u32 = 2;
    let id: u64 = window.id().into();
    let hwnd = HWND(id as isize);

    let factory = unsafe { CreateDXGIFactory1::<IDXGIFactory4>() }?;
    let adapter = get_hardware_adapter(&factory)?;

    let mut device: Option<ID3D12Device> = None;
    unsafe { D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_11_0, &mut device) }?;
    let device = device.unwrap();

    let command_queue = unsafe {
        device.CreateCommandQueue::<ID3D12CommandQueue>(&D3D12_COMMAND_QUEUE_DESC {
            Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
            Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
            ..Default::default()
        })
    }?;

    let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
        BufferCount: FRAME_COUNT,
        Width: window.inner_size().width,
        Height: window.inner_size().height,
        Format: DXGI_FORMAT_R8G8B8A8_UNORM,
        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
        SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            ..Default::default()
        },
        ..Default::default()
    };

    let swap_chain: IDXGISwapChain3 = unsafe {
        factory.CreateSwapChainForHwnd(&command_queue, hwnd, &swap_chain_desc, None, None)?
    }
    .cast()?;

    let frame_index = unsafe { swap_chain.GetCurrentBackBufferIndex() };

    let rtv_heap: ID3D12DescriptorHeap = unsafe {
        device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
            NumDescriptors: FRAME_COUNT,
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            ..Default::default()
        })
    }?;

    let rtv_descriptor_size =
        unsafe { device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV) } as usize;

    let rtv_handle = D3D12_CPU_DESCRIPTOR_HANDLE {
        ptr: unsafe { rtv_heap.GetCPUDescriptorHandleForHeapStart() }.ptr
            + frame_index as usize * rtv_descriptor_size,
    };

    let render_targets: Vec<ID3D12Resource> = {
        let mut render_targets = vec![];
        for i in 0..FRAME_COUNT {
            let render_target: ID3D12Resource = unsafe { swap_chain.GetBuffer(i)? };
            unsafe {
                device.CreateRenderTargetView(
                    &render_target,
                    None,
                    D3D12_CPU_DESCRIPTOR_HANDLE {
                        ptr: rtv_handle.ptr + i as usize * rtv_descriptor_size,
                    },
                )
            };
            render_targets.push(render_target);
        }
        render_targets
    };

    let backend_context = BackendContext {
        adapter,
        device: device.clone(),
        queue: command_queue,
        memory_allocator: None,
        protected_context: Protected::No,
    };

    let mut context = unsafe { DirectContext::new_d3d(&backend_context, None).unwrap() };

    let mut surfaces = render_targets
        .iter()
        .map(|render_target| {
            let backend_render_target = BackendRenderTarget::new_d3d(
                (
                    window.inner_size().width as i32,
                    window.inner_size().height as i32,
                ),
                &TextureResourceInfo {
                    resource: render_target.clone(),
                    alloc: None,
                    resource_state: D3D12_RESOURCE_STATE_COMMON,
                    format: DXGI_FORMAT_R8G8B8A8_UNORM,
                    sample_count: 1,
                    level_count: 0,
                    sample_quality_pattern: DXGI_STANDARD_MULTISAMPLE_QUALITY_PATTERN,
                    protected: Protected::No,
                },
            );

            surfaces::wrap_backend_render_target(
                &mut context,
                &backend_render_target,
                SurfaceOrigin::BottomLeft,
                ColorType::RGBA8888,
                None,
                None,
            )
            .ok_or(anyhow::anyhow!("wrap_backend_render_target failed"))
        })
        .collect::<Result<Vec<_>>>()?;

    fn get_hardware_adapter(factory: &IDXGIFactory4) -> Result<IDXGIAdapter1> {
        for i in 0.. {
            let adapter = unsafe { factory.EnumAdapters1(i)? };

            let mut desc = Default::default();
            unsafe { adapter.GetDesc1(&mut desc)? };

            if (DXGI_ADAPTER_FLAG(desc.Flags as i32) & DXGI_ADAPTER_FLAG_SOFTWARE)
                != DXGI_ADAPTER_FLAG_NONE
            {
                // Don't select the Basic Render Driver adapter.
                continue;
            }

            // Check to see whether the adapter supports Direct3D 12, but don't create the actual
            // device yet.
            if unsafe {
                D3D12CreateDevice(
                    &adapter,
                    D3D_FEATURE_LEVEL_11_0,
                    ptr::null_mut::<Option<ID3D12Device>>(),
                )
            }
            .is_ok()
            {
                return Ok(adapter);
            }
        }

        unreachable!()
    }

    let mut skia_context = context;

    println!("Skia initialized with {} surfaces.", surfaces.len());
    println!("Use Arrow Keys to move the rectangle.");

    let mut next_surface_index = 0;

    struct State {
        x: f32,
        y: f32,
    }

    let mut render = |state: &State| {
        let this_index = next_surface_index;
        next_surface_index = (next_surface_index + 1) % surfaces.len();

        let surface = &mut surfaces[this_index];
        let canvas = surface.canvas();

        // canvas.clear(Color::BLUE);

        // let mut paint = Paint::default();
        // paint.set_color(Color::RED);
        // paint.set_style(paint::Style::StrokeAndFill);
        // paint.set_anti_alias(true);
        // paint.set_stroke_width(10.0);

        // canvas.draw_rect(Rect::from_xywh(state.x, state.y, 200.0, 200.0), &paint);
        draw(&canvas);
        skia_context.flush_surface(surface);

        skia_context.submit(None);

        unsafe { swap_chain.Present(1, 0).ok().unwrap() };

        // NOTE: If you get some error when you render, you can check it with:
        // unsafe {
        //     device.GetDeviceRemovedReason().ok().unwrap();
        // }
    };

    enum ControlFlow {
        Continue,
        Exit,
    }

    use ControlFlow::*;

    let mut handle_event = |event, state: &mut State| match event {
        WindowEvent::RedrawRequested => {
            render(state);
            Continue
        }
        WindowEvent::KeyboardInput { event, .. } => {
            match event.logical_key {
                Key::Named(NamedKey::ArrowLeft) => state.x -= 10.0,
                Key::Named(NamedKey::ArrowRight) => state.x += 10.0,
                Key::Named(NamedKey::ArrowUp) => state.y += 10.0,
                Key::Named(NamedKey::ArrowDown) => state.y -= 10.0,
                Key::Named(NamedKey::Escape) => return Exit,
                _ => {}
            }

            render(state);
            Continue
        }
        WindowEvent::CloseRequested => Exit,
        _ => Continue,
    };

    let mut state = State { x: 100.0, y: 100.0 };

    event_loop.run(move |event, window| {
        if let Event::WindowEvent { event, .. } = event {
            match handle_event(event, &mut state) {
                Continue => {}
                Exit => window.exit(),
            }
        }
    })?;

    Ok(())
}
