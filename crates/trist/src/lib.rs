#[cfg(feature = "skia")]
mod skia_backend;

#[cfg(feature = "skia")]
pub use skia_backend::{Drawer, DrawerEnv, DrawerState};

use kurbo::*;

use skia_safe::Canvas;
use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::Window};

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Copy, Debug)]
pub struct PaintId {
    id: usize,
    is_static: bool,
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Copy, Debug)]
pub struct FontId {
    id: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

fn map_err(r: Result<u8, std::num::ParseIntError>) -> Result<u8, String> {
    r.map_err(|e| format!("Error parsing hex: {}", e))
}

impl Color {
    pub fn gray(lightness: f32) -> Color {
        Self {
            r: lightness,
            g: lightness,
            b: lightness,
            a: 1.0,
        }
    }

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn mix(&self, rhs: Color, s: f32) -> Color {
        Color {
            r: (1.0 - s) * self.r + s * rhs.r,
            g: (1.0 - s) * self.g + s * rhs.g,
            b: (1.0 - s) * self.b + s * rhs.b,
            a: (1.0 - s) * self.a + s * rhs.a,
        }
    }

    pub fn alpha(&self, a: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }

    pub const CYAN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const MAGENTA: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub fn hex(hex: &str) -> Result<Color, String> {
        if hex.len() == 9 && hex.starts_with('#') {
            // #FFFFFFFF (Red Green Blue Alpha)
            Ok(Color {
                r: map_err(u8::from_str_radix(&hex[1..3], 16))? as f32 / 255.0,
                g: map_err(u8::from_str_radix(&hex[3..5], 16))? as f32 / 255.0,
                b: map_err(u8::from_str_radix(&hex[5..7], 16))? as f32 / 255.0,
                a: map_err(u8::from_str_radix(&hex[7..9], 16))? as f32 / 255.0,
            })
        } else if hex.len() == 7 && hex.starts_with('#') {
            // #FFFFFF (Red Green Blue)
            Ok(Color {
                r: map_err(u8::from_str_radix(&hex[1..3], 16))? as f32 / 255.0,
                g: map_err(u8::from_str_radix(&hex[3..5], 16))? as f32 / 255.0,
                b: map_err(u8::from_str_radix(&hex[5..7], 16))? as f32 / 255.0,
                a: 1.0,
            })
        } else {
            Err("Error parsing hex. Example of valid formats: #FFFFFF or #ffffffff".to_string())
        }
    }

    pub const fn hex_const(hex: &str) -> Color {
        // Can't do f32 arithmetic in a const fn, so use a lookup table.
        let lut = [
            0.000000, 0.003922, 0.007843, 0.011765, 0.015686, 0.019608, 0.023529, 0.027451,
            0.031373, 0.035294, 0.039216, 0.043137, 0.047059, 0.050980, 0.054902, 0.058824,
            0.062745, 0.066667, 0.070588, 0.074510, 0.078431, 0.082353, 0.086275, 0.090196,
            0.094118, 0.098039, 0.101961, 0.105882, 0.109804, 0.113725, 0.117647, 0.121569,
            0.125490, 0.129412, 0.133333, 0.137255, 0.141176, 0.145098, 0.149020, 0.152941,
            0.156863, 0.160784, 0.164706, 0.168627, 0.172549, 0.176471, 0.180392, 0.184314,
            0.188235, 0.192157, 0.196078, 0.200000, 0.203922, 0.207843, 0.211765, 0.215686,
            0.219608, 0.223529, 0.227451, 0.231373, 0.235294, 0.239216, 0.243137, 0.247059,
            0.250980, 0.254902, 0.258824, 0.262745, 0.266667, 0.270588, 0.274510, 0.278431,
            0.282353, 0.286275, 0.290196, 0.294118, 0.298039, 0.301961, 0.305882, 0.309804,
            0.313725, 0.317647, 0.321569, 0.325490, 0.329412, 0.333333, 0.337255, 0.341176,
            0.345098, 0.349020, 0.352941, 0.356863, 0.360784, 0.364706, 0.368627, 0.372549,
            0.376471, 0.380392, 0.384314, 0.388235, 0.392157, 0.396078, 0.400000, 0.403922,
            0.407843, 0.411765, 0.415686, 0.419608, 0.423529, 0.427451, 0.431373, 0.435294,
            0.439216, 0.443137, 0.447059, 0.450980, 0.454902, 0.458824, 0.462745, 0.466667,
            0.470588, 0.474510, 0.478431, 0.482353, 0.486275, 0.490196, 0.494118, 0.498039,
            0.501961, 0.505882, 0.509804, 0.513725, 0.517647, 0.521569, 0.525490, 0.529412,
            0.533333, 0.537255, 0.541176, 0.545098, 0.549020, 0.552941, 0.556863, 0.560784,
            0.564706, 0.568627, 0.572549, 0.576471, 0.580392, 0.584314, 0.588235, 0.592157,
            0.596078, 0.600000, 0.603922, 0.607843, 0.611765, 0.615686, 0.619608, 0.623529,
            0.627451, 0.631373, 0.635294, 0.639216, 0.643137, 0.647059, 0.650980, 0.654902,
            0.658824, 0.662745, 0.666667, 0.670588, 0.674510, 0.678431, 0.682353, 0.686275,
            0.690196, 0.694118, 0.698039, 0.701961, 0.705882, 0.709804, 0.713725, 0.717647,
            0.721569, 0.725490, 0.729412, 0.733333, 0.737255, 0.741176, 0.745098, 0.749020,
            0.752941, 0.756863, 0.760784, 0.764706, 0.768627, 0.772549, 0.776471, 0.780392,
            0.784314, 0.788235, 0.792157, 0.796078, 0.800000, 0.803922, 0.807843, 0.811765,
            0.815686, 0.819608, 0.823529, 0.827451, 0.831373, 0.835294, 0.839216, 0.843137,
            0.847059, 0.850980, 0.854902, 0.858824, 0.862745, 0.866667, 0.870588, 0.874510,
            0.878431, 0.882353, 0.886275, 0.890196, 0.894118, 0.898039, 0.901961, 0.905882,
            0.909804, 0.913725, 0.917647, 0.921569, 0.925490, 0.929412, 0.933333, 0.937255,
            0.941176, 0.945098, 0.949020, 0.952941, 0.956863, 0.960784, 0.964706, 0.968627,
            0.972549, 0.976471, 0.980392, 0.984314, 0.988235, 0.992157, 0.996078, 1.000000,
        ];
        let bytes = hex.as_bytes();
        Color {
            r: lut[hex_digit(bytes[1]) * 16 + hex_digit(bytes[2])],
            g: lut[hex_digit(bytes[3]) * 16 + hex_digit(bytes[4])],
            b: lut[hex_digit(bytes[5]) * 16 + hex_digit(bytes[6])],
            a: 1.0,
        }
    }
}

const fn hex_digit(x: u8) -> usize {
    (if x >= b'0' && x <= b'9' {
        x - b'0'
    } else if x >= b'a' && x <= b'f' {
        x - b'a' + 10
    } else if x >= b'A' && x <= b'F' {
        x - b'A' + 10
    } else {
        panic!("bad hex digit")
    }) as usize
}

#[derive(Debug, Clone, Copy)]
pub enum Paint {
    Color(Color),
}

#[derive(Debug, Clone, Copy)]
pub enum Width {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

#[derive(Debug, Clone, Copy)]
pub enum Weight {
    Invisible,
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    Semibold,
    Bold,
    ExtraBold,
    Black,
    ExtraBlack,
}

#[derive(Debug, Clone)]
pub struct Font {
    pub name: String,
    pub size: f64,
    pub weight: Weight,
    pub width: Width,
}

#[derive(Debug, Clone)]
pub enum DrawerError {
    NoPaint(PaintId),
    NoFont(FontId),
    CannotDraw,
    CannotCreatePaint(Paint),
    CannotCreateFont(Font),
}

pub trait TristDrawerState {
    // manage paints
    fn create_fast_paint(&mut self, paint: Paint) -> Result<PaintId, DrawerError>;
    fn create_static_paint(&mut self, paint: Paint) -> Result<PaintId, DrawerError>;
    fn remove_static_paint(&mut self, paint: PaintId) -> Result<(), DrawerError>;
    fn create_font(&mut self, font: Font) -> Result<FontId, DrawerError>;
    fn remove_font(&mut self, font: FontId) -> Result<(), DrawerError>;
    fn text_bounds(
        &self,
        text: &str,
        max_width: Option<f64>,
        font: FontId,
        size: f64,
    ) -> Result<Size, DrawerError>;
}

pub trait TristDrawer<T: TristDrawerState> {
    fn save(&mut self);
    fn restore(&mut self);

    fn translate(&mut self, point: Vec2);

    fn clip_shape(&mut self, shape: &impl Shape);
    fn draw_shape(&mut self, shape: &impl Shape, paint: PaintId);
    fn draw_text(
        &mut self,
        text: &str,
        x: f64,
        y: f64,
        max_width: Option<f64>,
        size: f64,
        font: FontId,
        paint: PaintId,
    );
    fn state(&mut self) -> &mut T;
    fn current_transform(&self) -> Vec2;
}

#[cfg(feature = "skia")]
pub trait TristBackend<S: TristDrawerState> {
    fn new<E>(window: winit::window::WindowBuilder, event_loop: &EventLoop<E>) -> Self;
    fn on_resize(&mut self, size: PhysicalSize<u32>);
    fn request_redraw(&mut self);
    fn prepare_draw(&mut self);
    fn get_drawer_state(&mut self) -> &mut S;
    fn get_drawer(&mut self) -> (&Canvas, &mut DrawerState);
    fn draw(&mut self);
    fn window(&mut self) -> &mut Window;
}

pub const FALLBACK_SERIF_FONT: FontId = FontId { id: 0 };
pub const FALLBACK_MONOSPACE_FONT: FontId = FontId { id: 1 };