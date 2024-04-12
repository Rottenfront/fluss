use std::{borrow::Cow, collections::VecDeque};

use iced_wgpu::{
    core::{
        image::Renderer as _, renderer::Quad as IQuad, svg::Renderer as _, text::Renderer as _,
        Border, Pixels, Rectangle, Renderer as CoreRenderer, Size as ISize,
    },
    graphics::{mesh::Renderer as MeshRenderer, Mesh, Viewport},
    wgpu, Backend, Primitive, Renderer as IcedRenderer, Settings,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub use iced_wgpu::{
    core::{
        image::{FilterMethod as ImageFilterMethod, Handle as ImageHandle},
        svg::Handle as SvgHandle,
        Color, Font, Shadow as IShadow, Text,
    },
    graphics::text::{Editor, Paragraph},
};
pub use kurbo::{
    Point, Rect, RoundedRect, RoundedRectRadii, Size, TranslateScale as Transform, Vec2,
};

/// A shadow.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Shadow {
    /// The color of the shadow.
    pub color: Color,

    /// The offset of the shadow.
    pub offset: Vec2,

    /// The blur radius of the shadow.
    pub blur_radius: f64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RectQuad {
    pub origin: Point,
    pub size: Size,
    pub radius: RoundedRectRadii,
    pub border_width: f64,
    pub color: Color,
    pub border_color: Color,
    pub shadow: Shadow,
}

impl RectQuad {
    pub fn new(origin: Point, size: Size) -> Self {
        Self {
            origin,
            size,
            ..Default::default()
        }
    }

    pub fn from_rect(rect: Rect) -> Self {
        Self {
            origin: rect.origin(),
            size: rect.size(),
            ..Default::default()
        }
    }

    pub fn with_radius(self, radius: f64) -> Self {
        Self {
            radius: RoundedRectRadii::from_single_radius(radius),
            ..self
        }
    }

    pub fn with_border_width(self, border_width: f64) -> Self {
        Self {
            border_width,
            ..self
        }
    }

    pub fn with_color(self, color: Color) -> Self {
        Self { color, ..self }
    }

    pub fn with_border_color(self, border_color: Color) -> Self {
        Self {
            border_color,
            ..self
        }
    }

    pub fn with_shadow(self, shadow: Shadow) -> Self {
        Self { shadow, ..self }
    }
}

pub struct Renderer {
    renderer: IcedRenderer,
    viewport: Viewport,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    texture_format: wgpu::TextureFormat,
    // window: softbuffer::Surface,
    // clip_mask: tiny_skia::Mask,
    // primitive_stack: VecDeque<Vec<Primitive>>,
    // background_color: Color,
    // max_age: u8,
    width: u32,
    height: u32,
    scale: f64,

    bounds: Vec<Rect>,
    transforms: Vec<Transform>,
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        let physical_size = window.inner_size();
        let viewport = Viewport::with_physical_size(
            ISize::new(physical_size.width, physical_size.height),
            window.scale_factor(),
        );
        // Initialize wgpu
        let default_backend = wgpu::Backends::PRIMARY;

        let backend = wgpu::util::backend_bits_from_env().unwrap_or(default_backend);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: backend,
            ..Default::default()
        });
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let (format, adapter, device, queue) = pollster::block_on(async {
            let adapter =
                wgpu::util::initialize_adapter_from_env_or_default(&instance, Some(&surface))
                    .await
                    .expect("Create adapter");

            let adapter_features = adapter.features();

            #[cfg(target_arch = "wasm32")]
            let needed_limits =
                wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());

            #[cfg(not(target_arch = "wasm32"))]
            let needed_limits = wgpu::Limits::default();

            let capabilities = surface.get_capabilities(&adapter);

            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        features: adapter_features & wgpu::Features::default(),
                        limits: needed_limits,
                    },
                    None,
                )
                .await
                .expect("Request device");

            (
                capabilities
                    .formats
                    .iter()
                    .copied()
                    .find(wgpu::TextureFormat::is_srgb)
                    .or_else(|| capabilities.formats.first().copied())
                    .expect("Get preferred format"),
                adapter,
                device,
                queue,
            )
        });

        surface.configure(
            &device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width: physical_size.width,
                height: physical_size.height,
                present_mode: wgpu::PresentMode::AutoVsync,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
            },
        );

        // Initialize scene and GUI controls
        let renderer = IcedRenderer::new(
            Backend::new(&adapter, &device, &queue, Settings::default(), format),
            Font::default(),
            Pixels(16.0),
        );

        Self {
            renderer,
            viewport,
            device,
            queue,
            surface,
            texture_format: format,

            width: physical_size.width,
            height: physical_size.height,
            scale: window.scale_factor(),

            bounds: Vec::new(),
            transforms: Vec::new(),
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>, scale_factor: f64) {
        self.width = size.width;
        self.height = size.height;
        self.scale = scale_factor;
        self.viewport =
            Viewport::with_physical_size(ISize::new(size.width, size.height), scale_factor);

        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                format: self.texture_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::AutoVsync,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
            },
        );
    }

    pub fn present(&mut self) {
        while !self.transforms.is_empty() {
            self.end_transformation();
        }

        while !self.bounds.is_empty() {
            self.end_layer();
        }

        match self.surface.get_current_texture() {
            Ok(frame) => {
                let mut encoder = self
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // And then iced on top
                self.renderer.with_primitives(|backend, primitive| {
                    backend.present::<&str>(
                        &self.device,
                        &self.queue,
                        &mut encoder,
                        None,
                        frame.texture.format(),
                        &view,
                        primitive,
                        &self.viewport,
                        &[],
                    );
                });
                // Then we submit the work
                self.queue.submit(Some(encoder.finish()));
                frame.present();
            }
            Err(error) => match error {
                wgpu::SurfaceError::OutOfMemory => {
                    panic!("Swapchain error: {error}. Rendering cannot continue.")
                }
                _ => {
                    return;
                }
            },
        }
    }

    pub fn size(&self) -> Size {
        (self.width as _, self.height as _).into()
    }

    pub fn clear(&mut self) {
        self.renderer.clear();
    }

    pub fn current_transform(&self) -> Transform {
        let mut transform = Transform::default();
        for trans in &self.transforms {
            transform *= trans.clone();
        }
        transform
    }

    pub fn fill_rect(&mut self, rect: RectQuad) {
        let quad = IQuad {
            bounds: Rectangle {
                x: rect.origin.x as _,
                y: rect.origin.y as _,
                width: rect.size.width as _,
                height: rect.size.height as _,
            },
            border: Border {
                color: rect.border_color,
                width: rect.border_width as _,
                radius: [
                    rect.radius.top_left as _,
                    rect.radius.top_right as _,
                    rect.radius.bottom_right as _,
                    rect.radius.bottom_left as _,
                ]
                .into(),
            },
            shadow: IShadow {
                color: rect.shadow.color,
                offset: iced_wgpu::core::Vector::new(
                    rect.shadow.offset.x as _,
                    rect.shadow.offset.y as _,
                ),
                blur_radius: rect.shadow.blur_radius as _,
            },
        };
        self.renderer.fill_quad(quad, rect.color);
    }

    pub fn start_layer(&mut self, bounds: Rect) {
        self.bounds.push(bounds);
        self.renderer.start_layer();
    }

    pub fn end_layer(&mut self) {
        let bounds = self.bounds.pop().expect("a layer should be recording");
        self.renderer.end_layer(iced_wgpu::core::Rectangle::new(
            (bounds.origin().x as _, bounds.origin().y as _).into(),
            (bounds.size().width as _, bounds.size().height as _).into(),
        ));
    }

    pub fn start_transformation(&mut self, transformation: Transform) {
        self.transforms.push(transformation);
        self.renderer.start_transformation();
    }

    pub fn end_transformation(&mut self) {
        let transformation = self
            .transforms
            .pop()
            .expect("a transformation should be recording");
        self.renderer.end_transformation(
            iced_wgpu::core::Transformation::translate(
                transformation.translation.x as _,
                transformation.translation.y as _,
            ) * iced_wgpu::core::Transformation::scale(transformation.scale as _),
        );
    }

    pub const ICON_FONT: Font = iced_wgpu::Renderer::ICON_FONT;
    pub const CHECKMARK_ICON: char = '\u{f00c}';
    pub const ARROW_DOWN_ICON: char = '\u{e800}';

    pub fn default_font(&self) -> Font {
        self.renderer.default_font()
    }

    pub fn default_size(&self) -> f64 {
        self.renderer.default_size().0 as _
    }

    pub fn load_font(&mut self, bytes: Cow<'static, [u8]>) {
        self.renderer.load_font(bytes);
    }

    pub fn fill_paragraph(
        &mut self,
        paragraph: &Paragraph,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        self.renderer.fill_paragraph(
            paragraph,
            (position.x as _, position.y as _).into(),
            color,
            clip_bounds,
        );
    }

    pub fn fill_editor(
        &mut self,
        editor: &Editor,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        self.renderer.fill_editor(
            editor,
            (position.x as _, position.y as _).into(),
            color,
            clip_bounds,
        );
    }

    pub fn fill_text(&mut self, text: Text, position: Point, color: Color, clip_bounds: Rectangle) {
        self.renderer.fill_text(
            text,
            (position.x as _, position.y as _).into(),
            color,
            clip_bounds,
        )
    }

    pub fn measure_image(&self, handle: &ImageHandle) -> Size {
        let size = self.renderer.measure_image(handle);
        (size.width as _, size.height as _).into()
    }

    pub fn draw_image(
        &mut self,
        handle: ImageHandle,
        filter_method: ImageFilterMethod,
        bounds: Rectangle,
    ) {
        self.renderer.draw_image(handle, filter_method, bounds)
    }

    pub fn measure_svg(&self, handle: &SvgHandle) -> Size {
        let size = self.renderer.measure_svg(handle);
        (size.width as _, size.height as _).into()
    }

    pub fn draw_svg(&mut self, handle: SvgHandle, color: Option<Color>, bounds: Rect) {
        self.renderer.draw_svg(
            handle,
            color,
            Rectangle::new(
                (bounds.x0 as _, bounds.y0 as _).into(),
                ((bounds.x1 - bounds.x0) as _, (bounds.y1 - bounds.y0) as _).into(),
            ),
        );
    }

    pub fn draw_mesh(&mut self, mesh: Mesh) {
        self.renderer.draw_mesh(mesh)
    }
}
