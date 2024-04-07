use std::borrow::Cow;

use iced_wgpu::{
    core::{
        image::Renderer as _, renderer::Quad as IQuad, svg::Renderer as _, text::Renderer as _,
        Border, Rectangle, Renderer as CoreRenderer, Size as ISize,
    },
    graphics::{mesh::Renderer as MeshRenderer, Mesh, Primitive, Viewport},
    primitive::Custom,
    wgpu, Renderer as IcedRenderer,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub use iced_wgpu::{
    core::{
        image::{FilterMethod as ImageFilterMethod, Handle as ImageHandle},
        svg::Handle as SvgHandle,
        Color, Font, Shadow, Text,
    },
    graphics::text::{Editor, Paragraph},
};
pub use kurbo::{
    Point, Rect, RoundedRect, RoundedRectRadii, Size, TranslateScale as Transform, Vec2,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct RectQuad {
    origin: Point,
    size: Size,
    radius: RoundedRectRadii,
    border_width: f64,
    color: Color,
    border_color: Color,
    shadow: Shadow,
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
    pub renderer: IcedRenderer,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,

    pub width: u32,
    pub height: u32,
    pub scale: f64,

    bounds: Vec<Rect>,
    transforms: Vec<Transform>,
}

impl Renderer {
    pub async fn new(window: &Window) -> Self {
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = unsafe { instance.create_surface(window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let size = window.inner_size();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
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
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        let settings = iced_wgpu::Settings {
            ..Default::default()
        };
        let font = settings.default_font.clone();
        let font_size = settings.default_text_size;

        let backend = iced_wgpu::Backend::new(
            &adapter,
            &device,
            &queue,
            settings,
            wgpu::TextureFormat::Bgra8Unorm,
        );

        let renderer = iced_wgpu::Renderer::new(backend, font, font_size);
        Self {
            renderer,
            device,
            queue,
            surface,
            config,

            width: size.width,
            height: size.height,
            scale: 1.0,

            bounds: Vec::new(),
            transforms: Vec::new(),
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.width = size.width;
        self.height = size.height;
        println!("Resizing to {}x{}", size.width, size.height);
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn size(&self) -> Size {
        (self.width as _, self.height as _).into()
    }

    pub fn present(&mut self) {
        while !self.transforms.is_empty() {
            self.end_transformation();
        }

        while !self.bounds.is_empty() {
            self.end_layer();
        }

        let output = match self.surface.get_current_texture() {
            Ok(output) => output,
            Err(_) => {
                return;
            }
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.renderer.with_primitives(|backend, primitives| {
            backend.present::<&str>(
                &self.device,
                &self.queue,
                &mut encoder,
                None,
                wgpu::TextureFormat::Bgra8Unorm,
                &view,
                primitives,
                &Viewport::with_physical_size(ISize::new(self.width, self.height), self.scale),
                &[],
            );
            &[] as &[Primitive<Custom>]
        });
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
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
            shadow: rect.shadow,
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
