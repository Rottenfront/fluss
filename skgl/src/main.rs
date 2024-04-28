mod renderer {
    // Author: Alberto González Palomo https://sentido-labs.com
    // ©2019 Alberto González Palomo https://sentido-labs.com
    // Released under the MIT license: https://opensource.org/licenses/MIT
    #![allow(unknown_lints)]
    #![allow(clippy::unusual_byte_groupings)]
    use skia_safe::{
        gradient_shader, Color, Matrix, Paint, PaintJoin, PaintStyle, Path, Point, TileMode,
    };
    use std::cmp::min;

    const PI: f32 = std::f32::consts::PI;
    const DEGREES_IN_RADIANS: f32 = PI / 180.0;
    const PEN_SIZE: f32 = 1.0;

    fn point_in_circle(center: (f32, f32), radius: f32, radians: f32) -> (f32, f32) {
        (
            center.0 + radius * radians.cos(),
            center.1 - radius * radians.sin(),
        )
    }

    pub fn render_frame(
        frame: usize,
        fps: usize,
        bpm: usize,
        canvas: &skia_safe::canvas::Canvas,
    ) -> usize {
        let step = 12.0 * bpm as f32 / 60.0 / fps as f32;
        let frame_count = (360.0 / step) as usize;

        let size = {
            let dim = canvas.image_info().dimensions();
            min(dim.width, dim.height)
        };

        let center = (size / 2, size / 2);
        let chain_ring_radius = size / 2 * 100 / 100;
        let triangle_radius = size / 2 * 53 / 100;

        let rotation = frame as f32 * step;
        chain_ring(canvas, center, chain_ring_radius, rotation, 32);

        let triangle_rotation = 60.0 + rotation;
        triangle(
            canvas,
            center,
            triangle_radius,
            triangle_rotation,
            Some(0),
            Color::GREEN,
            true,
        );
        triangle(
            canvas,
            center,
            triangle_radius,
            triangle_rotation,
            Some(1),
            Color::BLUE,
            true,
        );
        triangle(
            canvas,
            center,
            triangle_radius,
            triangle_rotation,
            Some(2),
            Color::RED,
            true,
        );
        triangle(
            canvas,
            center,
            triangle_radius,
            triangle_rotation,
            Some(0),
            Color::YELLOW,
            false,
        );
        triangle(
            canvas,
            center,
            triangle_radius,
            triangle_rotation,
            Some(1),
            Color::CYAN,
            false,
        );
        triangle(
            canvas,
            center,
            triangle_radius,
            triangle_rotation,
            Some(2),
            Color::MAGENTA,
            false,
        );

        triangle(
            canvas,
            center,
            triangle_radius,
            triangle_rotation,
            None,
            Color::from(0x77_222222),
            true,
        );
        triangle(
            canvas,
            center,
            triangle_radius,
            triangle_rotation,
            None,
            Color::from(0x77_222222),
            false,
        );

        frame_count - (frame + 1)
    }

    fn chain_ring(
        canvas: &skia_safe::canvas::Canvas,
        center: (i32, i32),
        radius: i32,
        rotation: f32,
        teeth_count: i32,
    ) {
        canvas.save();
        canvas.translate(Point::from(center));
        canvas.save();
        canvas.rotate(rotation, None);

        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_stroke_width(PEN_SIZE.max(canvas.image_info().dimensions().width as f32 / 360.0));

        let center = (0, 0);
        let c = (center.0 as f32, center.1 as f32);
        let outer_radius = radius as f32;
        let inner_radius = outer_radius * 0.73;
        let ridge_radius = outer_radius * 0.85;
        let teeth_length = (outer_radius - ridge_radius) * 0.8;

        let delta = 2.0 * PI / (teeth_count as f32);
        let teeth_bottom_gap = 0.2 * delta;

        let mut alpha = PI / 2.0;
        let mut path = Path::new();
        for i in 0..teeth_count {
            let mut a = alpha - delta / 2.0 + teeth_bottom_gap / 2.0;
            let v = point_in_circle(c, outer_radius - teeth_length, a);
            if i == 0 {
                path.move_to(v);
            } else {
                path.line_to(v);
            }
            let middle = a + (delta - teeth_bottom_gap) / 2.0;
            a += delta - teeth_bottom_gap;
            path.cubic_to(
                point_in_circle(c, outer_radius * 1.035, middle),
                point_in_circle(c, outer_radius * 1.035, middle),
                point_in_circle(c, outer_radius - teeth_length, a),
            );
            a += teeth_bottom_gap;
            path.line_to(point_in_circle(c, outer_radius - teeth_length, a));

            alpha += delta;
        }
        path.close();

        let delta = -2.0 * PI / 5.0;
        let teeth_bottom_gap = 0.70 * delta;

        alpha = PI / 2.0;
        for i in 0..5 {
            let mut a = alpha - delta / 2.0 + teeth_bottom_gap / 2.0;
            let v = point_in_circle(c, inner_radius, a);
            if i == 0 {
                path.move_to(v);
            } else {
                path.line_to(v);
            }
            let middle = a + (delta - teeth_bottom_gap) / 2.0;
            a += delta - teeth_bottom_gap;
            path.cubic_to(
                point_in_circle(c, inner_radius - teeth_length * 1.33, middle),
                point_in_circle(c, inner_radius - teeth_length * 1.33, middle),
                point_in_circle(c, inner_radius, a),
            );
            a += teeth_bottom_gap;
            path.cubic_to(
                point_in_circle(c, inner_radius * 1.05, a - teeth_bottom_gap * 0.67),
                point_in_circle(c, inner_radius * 1.05, a - teeth_bottom_gap * 0.34),
                point_in_circle(c, inner_radius, a),
            );

            alpha += delta;
        }
        path.close();

        let bolt_radius = inner_radius * 0.81 * (delta - teeth_bottom_gap) / delta / PI;
        alpha = PI / 2.0;
        for _i in 0..5 {
            let c = point_in_circle(c, inner_radius + bolt_radius * 0.33, alpha);
            let mut a = alpha;
            for j in 0..5 {
                if j == 0 {
                    path.move_to(point_in_circle(c, bolt_radius, a));
                } else {
                    path.cubic_to(
                        point_in_circle(c, bolt_radius * 1.14, a + PI / 3.0),
                        point_in_circle(c, bolt_radius * 1.14, a + PI / 6.0),
                        point_in_circle(c, bolt_radius, a),
                    );
                }
                a -= PI / 2.0;
            }
            path.close();

            alpha += delta;
        }

        paint.set_style(PaintStyle::Fill);
        // Rust shade, from steel gray to rust color:
        paint.set_shader(gradient_shader::radial(
            (0.0, 0.04 * ridge_radius),
            ridge_radius,
            [Color::from(0xff_555555), Color::from(0xff_7b492d)].as_ref(),
            [0.8, 1.0].as_ref(),
            TileMode::Clamp,
            None,
            None,
        ));
        canvas.draw_path(&path, &paint);
        paint.set_shader(None); // Remove gradient.
        paint.set_style(PaintStyle::Stroke);
        paint.set_color(0xff_592e1f);
        canvas.draw_path(&path, &paint);

        canvas.restore();

        // Ridge around the chain ring, under the gear teeth:
        gradient(
            &mut paint,
            (0.0, -ridge_radius),
            (2.0 * ridge_radius, 2.0 * ridge_radius),
            (Color::from(0xff_592e1f), Color::from(0xff_885543)),
        );
        canvas.draw_circle(center, ridge_radius, &paint);

        canvas.restore();
    }

    #[allow(clippy::many_single_char_names)]
    fn triangle(
        canvas: &skia_safe::canvas::Canvas,
        center: (i32, i32),
        radius: i32,
        degrees: f32,
        vertex: Option<i32>,
        color: Color,
        wankel: bool,
    ) {
        let c = (center.0 as f32, center.1 as f32);
        let r = radius as f32;
        let b = r * 0.9;
        let delta = 120.0 * DEGREES_IN_RADIANS;
        let side = r / ((PI - delta) / 2.0).cos() * 2.0;

        let mut alpha = degrees * DEGREES_IN_RADIANS;
        let mut path = Path::new();
        let mut paint = Paint::default();
        match vertex {
            Some(index) => {
                let a = (degrees + (120 * index) as f32) * DEGREES_IN_RADIANS;
                let center = point_in_circle(c, r, a);
                let radii = match index {
                    0 | 2 => {
                        if wankel {
                            (0.36 * side, 0.404 * side)
                        } else {
                            (0.30 * side, 0.60 * side)
                        }
                    }
                    1 => {
                        if wankel {
                            (0.404 * side, 0.50 * side)
                        } else {
                            (0.420 * side, 0.50 * side)
                        }
                    }
                    i => panic!("Invalid vertex index {i} for triangle."),
                };
                gradient(&mut paint, center, radii, (color, Color::from(0x00_0000ff)))
            }
            None => {
                paint.set_anti_alias(true);
                paint.set_stroke_width(
                    PEN_SIZE.max(canvas.image_info().dimensions().width as f32 / 360.0),
                );
                paint.set_style(PaintStyle::Stroke);
                paint.set_stroke_join(PaintJoin::Bevel);
                // Highlight reflection on the top triangle edge:
                paint.set_shader(gradient_shader::radial(
                    (c.0, c.1 - 0.5 * r),
                    0.5 * r,
                    [Color::from(0xff_ffffff), color].as_ref(),
                    None,
                    TileMode::Clamp,
                    None,
                    None,
                ));
            }
        };
        for i in 0..4 {
            let v = point_in_circle(c, r, alpha);
            if i == 0 {
                path.move_to(v);
            } else if wankel {
                path.cubic_to(
                    point_in_circle(c, b, alpha - 2.0 * delta / 3.0),
                    point_in_circle(c, b, alpha - delta / 3.0),
                    v,
                );
            } else {
                path.line_to(v);
            }
            alpha += delta;
        }
        path.close();
        canvas.draw_path(&path, &paint);
    }

    fn gradient(paint: &mut Paint, center: (f32, f32), radii: (f32, f32), colors: (Color, Color)) {
        let mut matrix = Matrix::scale((1.0, radii.1 / radii.0));
        matrix.post_translate((center.0, center.1));
        #[allow(clippy::tuple_array_conversions)]
        paint.set_shader(gradient_shader::radial(
            (0.0, 0.0),
            radii.0,
            [colors.0, colors.1].as_ref(),
            None,
            TileMode::Clamp,
            None,
            &matrix,
        ));
    }
}

fn main() {
    use std::{
        ffi::CString,
        num::NonZeroU32,
        time::{Duration, Instant},
    };

    use gl::types::*;
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

    let mut gr_context = skia_safe::gpu::DirectContext::new_gl(interface, None)
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
        let expected_frame_length_seconds = 1.0 / 20.0;
        let frame_duration = Duration::from_secs_f32(expected_frame_length_seconds);

        if frame_start - previous_frame_start > frame_duration {
            draw_frame = true;
            previous_frame_start = frame_start;
        }
        if draw_frame {
            frame += 1;
            let canvas = env.surface.canvas();
            canvas.clear(Color::WHITE);
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
