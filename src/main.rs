use std::{any::Any, error::Error};

use flo_binding::{bind, Binding, Bound};
use time::Instant;

use shell::{
    kurbo::{Affine, Line, Rect, Size, Vec2},
    piet::{
        Color, FontFamily, FontStyle, FontWeight, ImageFormat, InterpolationMode, Piet,
        RenderContext, Text, TextLayout, TextLayoutBuilder,
    },
    Application, KbKey, KeyEvent, MouseButton, Region, WinHandler, WindowBuilder, WindowHandle,
};

const BG_COLOR: Color = Color::rgb8(0x27, 0x28, 0x22);
const FG_COLOR: Color = Color::rgb8(0xf0, 0xf0, 0xea);
const RED: Color = Color::rgb8(0xff, 0x80, 0x80);
const CYAN: Color = Color::rgb8(0x80, 0xff, 0xff);

use std::{collections::HashMap, hash::Hash};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ViewId(usize);

impl Hash for ViewId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.0);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    offset: Affine,
    size: Size,
}

impl Layout {
    pub fn new(offset: Affine, size: Size) -> Self {
        Self { offset, size }
    }
}

pub struct Context {
    arena: HashMap<ViewId, Box<dyn View>>,
    layouts: HashMap<ViewId, Layout>,
    last_id: usize,
    pointer: Vec2,
    pressed_mb: HashMap<MouseButton, bool>,
    pressed_keys: HashMap<KbKey, bool>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            arena: HashMap::new(),
            layouts: HashMap::new(),
            last_id: 0,
            pointer: Vec2::new(0.0, 0.0),
            pressed_mb: HashMap::new(),
            pressed_keys: HashMap::new(),
        }
    }

    pub fn push_view<V: View + 'static>(&mut self, view: V) -> ViewId {
        self.last_id += 1;
        self.arena.insert(ViewId(self.last_id), Box::new(view));
        ViewId(self.last_id)
    }

    pub fn get_view(&mut self, id: ViewId) -> Option<Box<dyn View>> {
        self.arena.remove(&id)
    }

    pub fn return_view(&mut self, id: ViewId, view: Box<dyn View>) {
        self.arena.insert(id, view);
    }
}

pub enum Event {
    /// being called every frame
    Update,
    MousePress(MouseButton),
    MouseUnpress(MouseButton),
}

pub trait View {
    fn draw(&self, id: ViewId, drawer: &mut Piet, max_size: Size, ctx: &mut Context);

    /// true if processed
    fn process_event(&mut self, event: &Event, ctx: &mut Context) -> bool;

    fn get_min_size(&self, drawer: &mut Piet) -> Size;

    fn is_flexible(&self) -> bool;
}

pub struct Font {
    family: FontFamily,
    weight: FontWeight,
    style: FontStyle,
}

impl Font {
    pub const SYSTEM: Self = Self {
        family: FontFamily::SYSTEM_UI,
        weight: FontWeight::NORMAL,
        style: FontStyle::Regular,
    };

    pub const SANS_SERIF: Self = Self {
        family: FontFamily::SANS_SERIF,
        weight: FontWeight::NORMAL,
        style: FontStyle::Regular,
    };

    pub const SERIF: Self = Self {
        family: FontFamily::SERIF,
        weight: FontWeight::NORMAL,
        style: FontStyle::Regular,
    };

    pub const MONOSPACE: Self = Self {
        family: FontFamily::SANS_SERIF,
        weight: FontWeight::NORMAL,
        style: FontStyle::Regular,
    };

    pub fn new(family: FontFamily) -> Self {
        Self {
            family,
            weight: FontWeight::NORMAL,
            style: FontStyle::Regular,
        }
    }

    pub fn with_style(self, style: FontStyle) -> Self {
        Self {
            family: self.family,
            weight: self.weight,
            style,
        }
    }
}

pub struct TextView {
    text: fn() -> String,
    color: Binding<Color>,
    size: Binding<f64>,
    font: Binding<FontFamily>,
}

impl TextView {
    pub fn new(
        text: fn() -> String,
        color: Binding<Color>,
        size: Binding<f64>,
        font: Binding<FontFamily>,
    ) -> Self {
        Self {
            color,
            text,
            font,
            size,
        }
    }
}

impl View for TextView {
    fn draw(&self, id: ViewId, drawer: &mut Piet, max_size: Size, ctx: &mut Context) {
        let offset = drawer.current_transform();
        let text = (self.text)();
        let font_size = self.size.get();
        let font = self.font.get();
        let color = self.color.get();
        let layout = drawer
            .text()
            .new_text_layout(text)
            .font(font, font_size)
            .text_color(color)
            .build()
            .unwrap();
        let size = layout.size();

        let size = Size::new(size.width, size.height.min(max_size.height));
        if let Some(layout) = ctx.layouts.get_mut(&id) {
            *layout = Layout::new(offset, size);
        } else {
            ctx.layouts.insert(id, Layout::new(offset, size));
        }

        drawer.draw_text(&layout, (0.0, 0.0));
    }

    fn process_event(&mut self, _event: &Event, _ctx: &mut Context) -> bool {
        false
    }

    fn get_min_size(&self, drawer: &mut Piet) -> Size {
        let text = (self.text)();
        let font_size = self.size.get();
        let font = self.font.get();
        drawer
            .text()
            .new_text_layout(text)
            .font(font, font_size)
            .build()
            .unwrap()
            .size()
    }

    fn is_flexible(&self) -> bool {
        true
    }
}

pub struct Filler {
    color: fn() -> Color,
}

impl Filler {
    pub fn new(color: fn() -> Color) -> Self {
        Self { color }
    }
}

impl View for Filler {
    fn draw(&self, id: ViewId, drawer: &mut Piet, max_size: Size, ctx: &mut Context) {
        let offset = drawer.current_transform();
        if let Some(layout) = ctx.layouts.get_mut(&id) {
            *layout = Layout::new(offset, max_size);
        } else {
            ctx.layouts.insert(id, Layout::new(offset, max_size));
        }
        let color = (self.color)();
        drawer.fill(&Rect::from_origin_size((0.0, 0.0), max_size), &color);
    }

    fn process_event(&mut self, _event: &Event, _ctx: &mut Context) -> bool {
        false
    }

    fn get_min_size(&self, drawer: &mut Piet) -> Size {
        Size::default()
    }

    fn is_flexible(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackDirection {
    Vertical,
    Horizontal,
    Depth,
}

pub struct Stack {
    direction: Binding<StackDirection>,
    views: Binding<Vec<ViewId>>,
}

impl Stack {
    pub fn vstack(views: Binding<Vec<ViewId>>) -> Self {
        Self {
            direction: bind(StackDirection::Vertical),
            views,
        }
    }

    pub fn hstack(views: Binding<Vec<ViewId>>) -> Self {
        Self {
            direction: bind(StackDirection::Horizontal),
            views,
        }
    }

    pub fn zstack(views: Binding<Vec<ViewId>>) -> Self {
        Self {
            direction: bind(StackDirection::Depth),
            views,
        }
    }
}

impl View for Stack {
    fn draw(&self, id: ViewId, drawer: &mut Piet, max_size: Size, ctx: &mut Context) {
        let offset = drawer.current_transform();
        if let Some(layout) = ctx.layouts.get_mut(&id) {
            *layout = Layout::new(offset, max_size);
        } else {
            ctx.layouts.insert(id, Layout::new(offset, max_size));
        }
        let views = self.views.get();
        if views.is_empty() {
            return;
        }
        match self.direction.get() {
            StackDirection::Vertical => {
                let height = max_size.height / (views.len() as f64);
                let mut current_offset = 0.0;
                for id in views {
                    let view = match ctx.get_view(id) {
                        None => continue,
                        Some(view) => view,
                    };
                    drawer.transform(Affine::translate((0.0, current_offset)));
                    view.draw(id, drawer, Size::new(max_size.width, height), ctx);
                    current_offset += height;
                    drawer.restore();
                    ctx.return_view(id, view);
                }
            }
            StackDirection::Horizontal => {
                let width = max_size.width / (views.len() as f64);
                let mut current_offset = 0.0;
                for id in views {
                    let view = match ctx.get_view(id) {
                        None => continue,
                        Some(view) => view,
                    };
                    drawer.transform(Affine::translate((current_offset, 0.0)));
                    view.draw(id, drawer, Size::new(width, max_size.height), ctx);
                    current_offset += width;
                    drawer.restore();
                    ctx.return_view(id, view);
                }
            }
            StackDirection::Depth => {
                for id in views {
                    let view = match ctx.get_view(id) {
                        None => continue,
                        Some(view) => view,
                    };
                    view.draw(id, drawer, max_size, ctx);
                    ctx.return_view(id, view);
                }
            }
        }
    }

    fn process_event(&mut self, _event: &Event, _ctx: &mut Context) -> bool {
        false
    }

    fn get_min_size(&self, drawer: &mut Piet) -> Size {
        Size::default()
    }

    fn is_flexible(&self) -> bool {
        true
    }
}

pub struct NullView;

impl View for NullView {
    fn draw(&self, id: ViewId, drawer: &mut Piet, max_size: Size, ctx: &mut Context) {}

    fn process_event(&mut self, event: &Event, ctx: &mut Context) -> bool {
        false
    }

    fn get_min_size(&self, drawer: &mut Piet) -> Size {
        Size::new(0.0, 0.0)
    }

    fn is_flexible(&self) -> bool {
        true
    }
}

struct UIApp<V: View + 'static> {
    handle: WindowHandle,
    size: Size,
    start_time: Instant,
    last_time: Instant,
    view: V,
    ctx: Context,
    title: Binding<String>,
    current_title: String,
}

impl<V: View + 'static> WinHandler for UIApp<V> {
    fn connect(&mut self, handle: &WindowHandle) {
        self.handle = handle.clone();
    }

    fn prepare_paint(&mut self) {
        let Self {
            ref mut handle,
            ref mut size,
            ref mut view,
            ref mut ctx,
            ref mut title,
            ref mut current_title,
            ..
        } = self;
        view.process_event(&Event::Update, ctx);
        let new_title = title.get();
        if new_title != *current_title {
            handle.set_title(&new_title);
        }
        *current_title = new_title;
    }

    fn paint(&mut self, piet: &mut Piet, _: &Region) {
        let rect = self.size.to_rect();
        piet.fill(rect, &BG_COLOR);

        self.view.draw(ViewId(0), piet, self.size, &mut self.ctx);

        let now = Instant::now();
        let msg = format!("{}ms", (now - self.last_time).whole_milliseconds());

        self.last_time = now;
        let layout = piet
            .text()
            .new_text_layout(msg)
            .font(FontFamily::MONOSPACE, 14.0)
            .text_color(FG_COLOR)
            .build()
            .unwrap();

        piet.draw_text(&layout, (0.0, 0.0));
        self.handle.request_anim_frame();
    }

    fn command(&mut self, id: u32) {
        match id {
            0x100 => self.handle.close(),
            _ => println!("unexpected id {id}"),
        }
    }

    fn key_down(&mut self, event: KeyEvent) -> bool {
        println!("keydown: {event:?}");
        false
    }

    fn size(&mut self, size: Size) {
        self.size = size;
    }

    fn request_close(&mut self) {
        self.handle.close();
    }

    fn destroy(&mut self) {
        Application::global().quit()
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

fn run<V: View + 'static, F: Fn(&mut Context) -> V>(view: F, title: Binding<String>) {
    tracing_subscriber::fmt().init();
    let app = Application::new().unwrap();
    let mut builder = WindowBuilder::new(app.clone());
    let mut ctx = Context::new();
    let view = view(&mut ctx);
    let current_title = title.get();
    let uiapp = UIApp {
        size: Size::ZERO,
        handle: Default::default(),
        start_time: time::Instant::now(),
        last_time: time::Instant::now(),
        view,
        ctx,
        title,
        current_title,
    };
    builder.set_handler(Box::new(uiapp));
    builder.set_title("Performance tester");

    let window = builder.build().unwrap();
    window.show();

    app.run(None);
}

fn main() {
    run(
        |ctx| {
            let first = {
                let first = ctx.push_view(Filler::new(|| Color::RED));
                let txt = ctx.push_view(TextView::new(
                    || "AAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA".into(),
                    bind(Color::BLACK),
                    bind(24.0),
                    bind(FontFamily::MONOSPACE),
                ));
                ctx.push_view(Stack::zstack(bind(vec![first, txt])))
            };
            let second = {
                let first = ctx.push_view(Filler::new(|| Color::GREEN));
                let txt = ctx.push_view(TextView::new(
                    || "AAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA".into(),
                    bind(Color::BLACK),
                    bind(24.0),
                    bind(FontFamily::MONOSPACE),
                ));
                ctx.push_view(Stack::zstack(bind(vec![first, txt])))
            };
            let third = {
                let first = ctx.push_view(Filler::new(|| Color::BLUE));
                let txt = ctx.push_view(TextView::new(
                    || "AAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAA".into(),
                    bind(Color::BLACK),
                    bind(24.0),
                    bind(FontFamily::MONOSPACE),
                ));
                ctx.push_view(Stack::zstack(bind(vec![first, txt])))
            };

            Stack::hstack(bind(vec![second, first, third]))
        },
        bind("wtf".to_string()),
    )
}
