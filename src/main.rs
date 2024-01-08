use std::cmp::min;
use std::path::PathBuf;

use skia_safe::font_style::{Slant, Weight, Width};
use skia_safe::gradient_shader::GradientShaderColors;
use skia_safe::{
    Canvas, ClipOp, Color, Color4f, EncodedImageFormat, Font, FontMgr, FontStyle, IRect, Image,
    ImageInfo, Paint, Point, Rect, Scalar, Shader, Size, TextBlob, TileMode,
};
use winit::{
    dpi::LogicalSize,
    event::{Event, KeyEvent, Modifiers, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::KeyCode,
    window::{Window, WindowBuilder},
};

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

mod public_api {
    use std::path::PathBuf;

    pub use kurbo::*;
    use winit::event::{KeyEvent, MouseButton};

    pub enum WidgetEvent {
        CursorMove((f32, f32)),
        CursorLeft,
        ButtonPress(MouseButton),
        ButtonRelease(MouseButton),
        Scroll {
            delta: (f32, f32),
        },
        KeyboardInput(KeyEvent),
        Resized((f32, f32)),
        Disabled,
        Enabled,

        /// A file has been dropped into the widget.
        ///
        /// When the user drops multiple files at once, this event will be emitted for each file
        /// separately.
        DroppedFile(PathBuf),

        /// A file is being hovered over the widget.
        ///
        /// When the user hovers multiple files at once, this event will be emitted for each file
        /// separately.
        HoveredFile(PathBuf),

        /// A file was hovered, but has exited the widget.
        ///
        /// There will be a single `HoveredFileCancelled` event triggered even if multiple files were
        /// hovered.
        HoveredFileCancelled,
    }

    pub type FontId = usize;
    pub const MONOSPACE_FONT: FontId = 0;
    pub const SERIF_FONT: FontId = 1;

    pub enum ImageFormat {
        Rgba,
    }
    pub struct Image {
        data: Vec<u8>,
        format: ImageFormat,
    }
    pub type ImageId = usize;

    pub enum BezierCurve {
        Linear(Point),
        Quad(Point, Point),
        Cubic(Point, Point, Point),
    }

    pub trait BezierPathTrait {
        fn line_to(&mut self, point: Point);
        fn quad_to(&mut self, point1: Point, point2: Point);
        fn cubic_to(&mut self, point1: Point, point2: Point, point3: Point);
    }

    impl BezierPathTrait for Vec<BezierCurve> {
        fn line_to(&mut self, point: Point) {
            self.push(BezierCurve::Linear(point));
        }
        fn quad_to(&mut self, point1: Point, point2: Point) {
            self.push(BezierCurve::Quad(point1, point2));
        }
        fn cubic_to(&mut self, point1: Point, point2: Point, point3: Point) {
            self.push(BezierCurve::Cubic(point1, point2, point3));
        }
    }

    // pub enum Primitive {
    //     Rect {
    //         top_left: Point,
    //         bottom_right: Point,
    //     },
    //     RoundedRect {
    //         top_left: Point,
    //         bottom_right: Point,
    //         radius: f32,
    //     },
    //     Text {
    //         text: String,
    //         top_left: Point,
    //         font: FontId,
    //     },
    //     Path {
    //         start: Point,
    //         path: Vec<BezierCurve>,
    //     },
    // }

    pub struct Color {
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    }

    pub enum Filler {
        Image(ImageId),
        Color(Color),
        LinearGradient((Point, Color), (Point, Color)),
    }

    pub trait Context {
        fn create_image(&mut self, img: Image) -> ImageId;
        fn release_image(&mut self, id: ImageId) -> Result<(), String>;
    }

    pub trait Element {
        /// `self` field is not mutable 'cause it's better to use bindings for drawing context
        ///
        /// Binding doesn't require mutability to modify content
        fn draw(&self, max_bound: Point) -> Vec<(Box<impl Shape>, Filler)>;
        /// Returns true if event is handled, false if event is passed
        fn handle_event<Ctx: Context>(&mut self, event: WidgetEvent, ctx: &mut Ctx) -> bool;
        /// Must be called by context on widget creation
        fn prepare<Ctx: Context>(&mut self, ctx: &mut Ctx);
        /// Must be called by context on widget deletion, can be used for releasing used data
        fn delete<Ctx: Context>(&mut self, ctx: &mut Ctx);
    }
}

pub enum AppFocus {
    LeftDock,
    RightDock,
    BottomDock,
    /// Splitted editor part number
    /// Iif there are no splits - can be any number
    Split(usize),
}

struct ApplicationState {
    monospace_font: Font,
    test_editor: EditorState,
}

struct Bar {
    hovered_button: isize,
    enabled_button: isize,
}

#[derive(Debug, Clone)]
pub enum SpanType {
    Text,
    Comment,
    Keyword,
}

#[derive(Debug, Clone, Copy)]
struct Cursor {
    selection_pos: (usize, usize),
    position: (usize, usize),
    /// Needed to represent normal position on line when cursor switches to the line with length
    /// less than cursor's position on line
    normal_x: usize,
}

pub struct EditorState {
    file: Vec<String>,
    name: String,
    path: Option<PathBuf>,
    scroll: (f32, f32),
    // spans: Binding<Vec<(usize, usize, usize, usize, SpanType)>>,
    cursors: Vec<Cursor>,
}
#[derive(PartialEq, Eq)]
pub enum Arrow {
    Left,
    Up,
    Right,
    Down,
}

const DISTANCE_BETWEEN_NUMBER_AND_LINE: f32 = 20.0;
pub const UPPER_BAR_HEIGHT: f32 = 40.0;
pub const LEFT_BAR_WIDTH: f32 = 40.0;
pub const RIGHT_BAR_WIDTH: f32 = 40.0;
pub const BOTTOM_BAR_HEIGHT: f32 = 30.0;

impl EditorState {
    fn draw(&self, canvas: &Canvas, rect: Rect, monospace_font: &Font) {
        let (font_height, metrics) = monospace_font.metrics();
        let x1 = rect.left;
        let x2 = rect.right;
        let y1 = rect.top;
        let y2 = rect.bottom;
        let clip = canvas.save();
        // canvas.draw_text_blob(
        //     TextBlob::new("Text", monospace_font).unwrap(),
        //     Point::new(100.0, 100.0),
        //     &Paint::new(Color4f::new(0.0, 0.0, 0.0, 1.0), None),
        // );
        canvas.clip_rect(&rect, Some(ClipOp::Intersect), Some(true));
        let first_line = (self.scroll.1 / font_height) as usize;
        let last_line = ((self.scroll.1 + (y2 - y1)) / font_height) as usize + 1;

        let number = format!("{}", self.file.len());
        let mut font_width = [0.0];
        monospace_font.get_widths(&[25], &mut font_width);
        let font_width = font_width[0];
        let number_len = number.len() as f32 * font_width;
        let x1 = DISTANCE_BETWEEN_NUMBER_AND_LINE + x1 + number_len;

        // Lines render
        // delta is distance between left-top corner of first displayed line and left-top corner of editor
        let delta_y = self.scroll.1 - font_height * (first_line as f32);
        let y1 = y1 - delta_y + (1 - first_line) as f32 * font_height;
        for i in first_line..min(last_line, self.file.len()) {
            // Render line
            canvas.draw_text_blob(
                TextBlob::new(&self.file[i], monospace_font).unwrap(),
                Point::new(
                    x1 + DISTANCE_BETWEEN_NUMBER_AND_LINE,
                    y1 + i as f32 * font_height,
                ),
                &Paint::new(Color4f::new(0.0, 0.0, 0.0, 1.0), None),
            );
            // Render number
            let linenum = format!("{i}");
            canvas.draw_text_blob(
                TextBlob::new(&linenum, monospace_font).unwrap(),
                Point::new(
                    x1 - linenum.len() as f32 * font_width,
                    y1 + i as f32 * font_height,
                ),
                &Paint::new(Color4f::new(0.0, 0.0, 0.0, 1.0), None),
            );
            // gc.begin_line_layout(
            //     x1 - DISTANCE_BETWEEN_NUMBER_AND_LINE,
            //     y1 + i as f32 * metrics.height,
            //     TextAlignment::Right,
            // );
            // gc.layout_text(MONOSPACE_FONT, format!("{}", i + 1));
            // gc.draw_text_layout();
        }
        for cursor in &self.cursors {
            let mut selection_line = cursor.selection_pos.0;
            let mut selection_char = cursor.selection_pos.1;
            let mut line = cursor.position.0;
            let mut ch = cursor.position.1;
            let x = x1 + ch as f32 * font_width + DISTANCE_BETWEEN_NUMBER_AND_LINE;
            let y = y1 + line as f32 * font_height + (font_height - metrics.cap_height) / 2.0;
            canvas.draw_rect(
                &Rect::from_ltrb(x - 1.0, y - font_height, x + 1.0, y),
                &Paint::new(Color4f::new(0.0, 0.0, 1.0, 1.0), None),
            );
        }
        canvas.restore();
    }

    pub fn handle_cursor_moved(&mut self, cursor_id: usize, arrow: Arrow) {
        let cursor = &mut self.cursors[cursor_id];
        match arrow {
            Arrow::Up => {
                if cursor.position.0 != 0 {
                    cursor.position.0 -= 1;
                    cursor.position.1 = min(self.file[cursor.position.0].len(), cursor.normal_x);
                }
            }
            Arrow::Down => {
                if cursor.position.0 < self.file.len() - 1 {
                    cursor.position.0 += 1;
                    cursor.position.1 = min(self.file[cursor.position.0].len(), cursor.normal_x);
                }
            }
            Arrow::Left | Arrow::Right => {
                if cursor.selection_pos != cursor.position {
                    // adjust selection to one of the sides
                    if arrow == Arrow::Left {
                        if cursor.selection_pos.0 < cursor.position.0
                            || (cursor.selection_pos.0 == cursor.position.0
                                && cursor.selection_pos.1 < cursor.position.1)
                        {
                            cursor.position = cursor.selection_pos;
                        } else {
                            cursor.selection_pos = cursor.position;
                        }
                    } else {
                        if cursor.selection_pos.0 < cursor.position.0
                            || (cursor.selection_pos.0 == cursor.position.0
                                && cursor.selection_pos.1 < cursor.position.1)
                        {
                            cursor.selection_pos = cursor.position;
                        } else {
                            cursor.position = cursor.selection_pos;
                        }
                    }
                } else {
                    // increase/decrease cursor index
                    if arrow == Arrow::Left {
                        if cursor.position.1 == 0 {
                            if cursor.position.0 != 0 {
                                cursor.position.0 -= 1;
                                cursor.position.1 = self.file[cursor.position.0].len();
                            }
                        } else {
                            cursor.position.1 -= 1;
                        }
                    } else {
                        if cursor.position.0 < self.file.len() {
                            if cursor.position.1 == self.file[cursor.position.0].len() {
                                if cursor.position.0 < self.file.len() - 1 {
                                    cursor.position.0 += 1;
                                    cursor.position.1 = 0;
                                }
                            } else {
                                cursor.position.1 += 1;
                            }
                        }
                    }
                }
                cursor.normal_x = cursor.position.1;
            }
        }
        cursor.selection_pos = cursor.position;
    }
}

impl ApplicationState {
    /// Renders a rectangle that occupies exactly half of the canvas
    fn draw(&self, canvas: &Canvas) {
        let canvas_size = Size::from(canvas.base_layer_size());

        canvas.clear(Color::WHITE);

        canvas.draw_rect(
            Rect::from_ltrb(0.0, 0.0, canvas_size.width, 40.0),
            &Paint::new(Color4f::new(0.0, 0.0, 1.0, 1.0), None),
        );
        // canvas.draw_rect(
        //     Rect::from_ltrb(0.0, 0.0, canvas_size.width, canvas_size.height),
        //     &Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None),
        // );
        self.test_editor.draw(
            canvas,
            Rect::from_ltrb(
                LEFT_BAR_WIDTH,
                UPPER_BAR_HEIGHT,
                canvas_size.width - RIGHT_BAR_WIDTH,
                canvas_size.height - BOTTOM_BAR_HEIGHT,
            ),
            &self.monospace_font,
        );

        // canvas.draw_rect(
        //     Rect::from_ltrb(0.0, 0.0, canvas_size.width, 30.0),
        //     &Paint::new(Color4f::new(0.0, 1.0, 0.0, 1.0), None),
        // );

        // let rect_size = canvas_size / 2.0;
        // let rect = Rect::from_point_and_size(
        //     Point::new(
        //         (canvas_size.width - rect_size.width) / 2.0,
        //         (canvas_size.height - rect_size.height) / 2.0,
        //     ),
        //     rect_size,
        // );
        // let scalars = [0.0f32, 1.0f32];
        // let mut paint = Paint::default();
        // paint.set_shader(Shader::linear_gradient(
        //     (
        //         Point::new(rect.left, rect.top),
        //         Point::new(rect.right, rect.bottom),
        //     ),
        //     GradientShaderColors::Colors(&[Color::RED, Color::BLUE]),
        //     None,
        //     TileMode::Repeat,
        //     None,
        //     None,
        // ));
        // canvas.draw_rect(rect, &paint);
    }

    // fn draw_left_bar(&self, canvas: &Canvas) {}
}

#[cfg(feature = "gl-render")]
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

    use skia_safe::{
        gpu::{self, backend_render_targets, gl::FramebufferInfo, SurfaceOrigin},
        ColorType, Surface,
    };
    let mut app = ApplicationState {
        monospace_font: Font::new(
            FontMgr::new()
                .match_family_style(
                    "Cascadia Code PL",
                    FontStyle::new(Weight::BOLD, Width::NORMAL, Slant::Upright),
                )
                .unwrap(),
            13.0,
        ),
        test_editor: EditorState {
            file: vec![
                "fn main() {".into(),
                "    println!(\"Hello, World!\");".into(),
                "}".into(),
            ],
            name: "name".to_string(),
            path: None,
            scroll: (0.0, 0.0),
            cursors: vec![Cursor {
                position: (0, 0),
                selection_pos: (0, 0),
                normal_x: 0,
            }],
        },
    };

    let el = EventLoop::new().expect("Failed to create event loop");
    let winit_window_builder = WindowBuilder::new()
        .with_title("rust-skia-gl-window")
        .with_inner_size(LogicalSize::new(800, 800))
        .with_transparent(true)
        .with_blur(true);

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
    let interface = gpu::gl::Interface::new_load_with(|name| {
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
        gr_context: &mut gpu::DirectContext,
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
    let mut key_pressed = false;

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
                    event:
                        KeyEvent {
                            logical_key,
                            physical_key,
                            repeat,
                            ..
                        },
                    ..
                } => {
                    if modifiers.state().super_key() && logical_key == "q" {
                        window_target.exit();
                    }
                    if !key_pressed || repeat {
                        if !modifiers.state().super_key() && !modifiers.state().control_key() {
                            let cursors = app.test_editor.cursors.clone();
                            for (id, cursor) in &mut cursors.iter().enumerate() {
                                // let mut start_line = cursor.0;
                                // let mut start_char = cursor.1;
                                // let mut line = cursor.2;
                                // let mut ch = cursor.3;
                                match physical_key {
                                    winit::keyboard::PhysicalKey::Code(code) => match code {
                                        KeyCode::ArrowUp => {
                                            app.test_editor.handle_cursor_moved(id, Arrow::Up);
                                        }
                                        KeyCode::ArrowDown => {
                                            app.test_editor.handle_cursor_moved(id, Arrow::Down);
                                        }
                                        KeyCode::ArrowLeft => {
                                            app.test_editor.handle_cursor_moved(id, Arrow::Left);
                                        }
                                        KeyCode::ArrowRight => {
                                            app.test_editor.handle_cursor_moved(id, Arrow::Right);
                                        }
                                        _ => {
                                            // Input character
                                        }
                                    },
                                    // winit::keyboard::PhysicalKey::Unidentified()
                                    _ => {}
                                }
                            }
                        }
                    }
                    if !repeat {
                        key_pressed = !key_pressed;
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
            app.draw(&canvas);
            // renderer::render_frame(frame % 360, 12, 60, canvas);
            env.gr_context.flush_and_submit();
            env.gl_surface.swap_buffers(&env.gl_context).unwrap();
        }

        window_target.set_control_flow(ControlFlow::WaitUntil(
            previous_frame_start + frame_duration,
        ))
    })
    .expect("run() failed");
}

#[cfg(all(feature = "d3d-render", not(feature = "gl-render")))]
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
