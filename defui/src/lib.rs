mod viewtuple;

use std::{collections::HashMap, hash::Hash};
pub use trist::*;
pub use viewtuple::*;

use std::time::{Duration, Instant};

use winit::{
    dpi::LogicalSize,
    event::{Event as WEvent, WindowEvent},
    event_loop::{EventLoop, EventLoopBuilder},
    window::WindowBuilder,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ViewId(usize);

impl Hash for ViewId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.0);
    }
}

pub struct Context {
    arena: HashMap<ViewId, Box<dyn View>>,
    layouts: HashMap<ViewId, RRect>,
    last_id: usize,
}

impl Context {
    pub fn new() -> Self {
        Context {
            arena: HashMap::new(),
            layouts: HashMap::new(),
            last_id: 0,
        }
    }

    pub fn new_id(&mut self) -> ViewId {
        self.last_id += 1;
        ViewId(self.last_id)
    }
}

pub enum Event {
    /// being called every frame
    Update,
}

pub trait View {
    fn draw(
        &self,
        id: ViewId,
        drawer: &mut Drawer,
        offset: TranslateScale,
        max_size: Size,
        ctx: &mut Context,
    );

    /// true if processed
    fn process_event(
        &mut self,
        event: &Event,
        draw_ctx: &mut DrawerState,
        ctx: &mut Context,
    ) -> bool;

    fn get_min_size(&self, draw_ctx: &DrawerState) -> Size;

    fn is_flexible(&self) -> bool;
}

enum WinitEvent {}

pub struct Application<T: View> {
    ctx: Context,
    view: T,
    env: DrawerEnv,
    el: EventLoop<WinitEvent>,
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
    fn draw(
        &self,
        id: ViewId,
        drawer: &mut Drawer,
        offset: TranslateScale,
        max_size: Size,
        ctx: &mut Context,
    ) {
        if let Some(layout) = ctx.layouts.get_mut(&id) {
            *layout = RRect::from_rect(
                Rect::from_origin_size(
                    offset.translation,
                    Size::new(
                        max_size.width * offset.scale_x,
                        max_size.height * offset.scale_y,
                    ),
                ),
                RectRadii::default(),
            );
        } else {
            ctx.layouts.insert(
                id,
                RRect::from_rect(
                    Rect::from_origin_size(
                        offset.translation,
                        Size::new(
                            max_size.width * offset.scale_x,
                            max_size.height * offset.scale_y,
                        ),
                    ),
                    RectRadii::default(),
                ),
            );
        }
        let color = (self.color)();
        let color = drawer
            .state()
            .create_fast_paint(Paint::Color(color))
            .unwrap();
        drawer.draw_rect(&ctx.layouts[&id], color)
    }

    fn process_event(
        &mut self,
        _event: &Event,
        _draw_ctx: &mut DrawerState,
        _ctx: &mut Context,
    ) -> bool {
        false
    }

    fn get_min_size(&self, _draw_ctx: &DrawerState) -> Size {
        Size::default()
    }

    fn is_flexible(&self) -> bool {
        true
    }
}

pub enum StackDirection {
    Vertical,
    Horizontal,
}

pub struct Stack {
    direction: StackDirection,
    views: Vec<(ViewId, Box<dyn View>)>,
}

impl Stack {
    pub fn vstack(views: Vec<(ViewId, Box<dyn View>)>) -> Self {
        Self {
            direction: StackDirection::Vertical,
            views,
        }
    }

    pub fn hstack(views: Vec<(ViewId, Box<dyn View>)>) -> Self {
        Self {
            direction: StackDirection::Horizontal,
            views,
        }
    }
}

impl View for Stack {
    fn draw(
        &self,
        id: ViewId,
        drawer: &mut Drawer,
        offset: TranslateScale,
        max_size: Size,
        ctx: &mut Context,
    ) {
        if let Some(layout) = ctx.layouts.get_mut(&id) {
            *layout = RRect::from_rect(
                Rect::from_origin_size(
                    offset.translation,
                    Size::new(
                        max_size.width * offset.scale_x,
                        max_size.height * offset.scale_y,
                    ),
                ),
                RectRadii::default(),
            );
        } else {
            ctx.layouts.insert(
                id,
                RRect::from_rect(
                    Rect::from_origin_size(
                        offset.translation,
                        Size::new(
                            max_size.width * offset.scale_x,
                            max_size.height * offset.scale_y,
                        ),
                    ),
                    RectRadii::default(),
                ),
            );
        }
        if self.views.is_empty() {
            return;
        }
        match self.direction {
            StackDirection::Vertical => {
                let height = max_size.height / (self.views.len() as f64);
                let mut current_offset = 0.0;
                for (id, view) in &self.views {
                    view.draw(
                        *id,
                        drawer,
                        offset * TranslateScale::new((0.0, current_offset).into(), 1.0, 1.0),
                        Size::new(max_size.width, height),
                        ctx,
                    );
                    current_offset += height;
                }
            }
            StackDirection::Horizontal => {
                let width = max_size.width / (self.views.len() as f64);
                let mut current_offset = 0.0;
                for (id, view) in &self.views {
                    view.draw(
                        *id,
                        drawer,
                        offset * TranslateScale::new((current_offset, 0.0).into(), 1.0, 1.0),
                        Size::new(width, max_size.height),
                        ctx,
                    );
                    current_offset = width;
                }
            }
        }
    }

    fn process_event(
        &mut self,
        _event: &Event,
        _draw_ctx: &mut DrawerState,
        _ctx: &mut Context,
    ) -> bool {
        false
    }

    fn get_min_size(&self, _draw_ctx: &DrawerState) -> Size {
        Size::default()
    }

    fn is_flexible(&self) -> bool {
        true
    }
}

impl<T: View> Application<T> {
    pub fn new(title: &str, view: fn(&mut Context, &mut DrawerState) -> T) -> Self {
        let el = EventLoopBuilder::<WinitEvent>::with_user_event()
            .build()
            .expect("Failed to create event loop");
        let builder = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(LogicalSize::new(800, 800))
            .with_transparent(true)
            .with_blur(true);

        let mut env = DrawerEnv::new(builder, &el);
        let mut context = Context::new();
        let view = view(&mut context, env.get_drawer_state());

        Self {
            ctx: context,
            view,
            env,
            el,
        }
    }

    pub fn run(self) {
        static EXPECTED_FRAME_DURATION: f32 = 1.0 / 60.0;
        let Self {
            mut view,
            mut env,
            el,
            mut ctx,
        } = self;
        let mut previous_frame_start = Instant::now();
        let frame_duration = Duration::from_secs_f32(EXPECTED_FRAME_DURATION);

        el.run(move |event, window_target| {
            let frame_start = Instant::now();
            let mut draw_frame = false;
            if let WEvent::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested => {
                        window_target.exit();
                        return;
                    }
                    WindowEvent::Resized(physical_size) => {
                        env.on_resize(physical_size);
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
                let size = env.window().inner_size();
                let (canvas, state) = env.get_drawer();
                view.process_event(&Event::Update, state, &mut ctx);
                let mut drawer = Drawer::new(canvas, state);
                drawer.clear(Color::TRANSPARENT);
                view.draw(
                    ViewId(0),
                    &mut drawer,
                    TranslateScale::default(),
                    Size::new(size.width as _, size.height as _),
                    &mut ctx,
                );
                drop(drawer);
                env.draw();
            }

            window_target.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                previous_frame_start + frame_duration,
            ))
        })
        .expect("run() failed");
    }
}
