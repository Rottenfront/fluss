use super::*;
use time::Instant;
use winit::window::Window;

pub struct UIApp<V: View + 'static> {
    window: Window,
    renderer: Renderer,
    root_view: V,
    ctx: Context,
    title: String,
    background_color: Color,
}

impl<V: View + 'static> UIApp<V> {
    pub fn new(
        event_loop: &EventLoop<()>,
        root_view: V,
        window_properties: WindowProperties,
    ) -> Self {
        let window = Window::new(&event_loop).unwrap();
        let renderer = Renderer::new(&window);
        Self {
            window,
            renderer,
            root_view,
            ctx: Context::new(),
            title: window_properties.title,
            background_color: window_properties.background_color,
        }
    }

    fn update_window_data(&mut self) {
        let actions = std::mem::take(&mut self.ctx.actions);
        for action in actions {
            match action {
                Action::SetTitle(title) => self.title = title,
                Action::SetCursor(cursor) => self.window.set_cursor_icon(cursor),
                Action::SetBackgroundColor(color) => self.background_color = color,
                Action::Quit => {}
            }
        }
    }

    fn clear_surface(&mut self) {
        self.renderer.clear();
    }

    fn draw_view(&mut self) {
        let size = self.renderer.size();
        println!("{:?}", size);
        self.root_view.draw(DrawContext {
            drawer: &mut self.renderer,
            size,
            ctx: &mut self.ctx,
        })
    }

    fn draw_debug_data(&mut self, before_draw_time: Instant) {
        // let now = Instant::now();
        // let full_frame = format!("{}ms", (now - self.last_time).whole_milliseconds());
        // let just_draw = format!("{}micros", (now - before_draw_time).whole_microseconds());

        // self.last_time = now;

        // let layout = piet
        //     .text()
        //     .new_text_layout(full_frame)
        //     .font(FontFamily::MONOSPACE, 14.0)
        //     .text_color(Color::WHITE)
        //     .build()
        //     .unwrap();

        // piet.draw_text(&layout, (0.0, 0.0));
        // let layout = piet
        //     .text()
        //     .new_text_layout(just_draw)
        //     .font(FontFamily::MONOSPACE, 14.0)
        //     .text_color(Color::WHITE)
        //     .build()
        //     .unwrap();

        // piet.draw_text(&layout, (100.0, 0.0));
    }

    fn after_draw(&mut self) {
        self.renderer.present();
    }

    pub fn request_redraw(&mut self) {
        self.window.request_redraw();
    }

    pub fn resize(&mut self) {
        self.renderer
            .resize(self.window.inner_size(), self.window.scale_factor());
    }

    pub fn paint(&mut self) {
        let before_draw = Instant::now();

        self.update_window_data();

        self.clear_surface();

        self.draw_view();

        self.draw_debug_data(before_draw);

        self.after_draw();
    }

    pub fn key_down(&mut self, event: KeyEvent) -> bool {
        println!("keydown: {event:?}");
        false
    }

    pub fn key_up(&mut self, event: KeyEvent) {
        println!("keyup: {event:?}");
    }

    pub fn wheel(&mut self, event: &ScrollEvent) {
        // println!("mouse_wheel {event:?}");
    }

    pub fn mouse_move(&mut self, event: &MouseMoveEvent) {
        self.ctx.pointer = event.pos.to_vec2();
    }

    pub fn mouse_down(&mut self, event: &MousePress) {
        self.ctx.pressed_mb.insert(event.button, true);
        self.root_view.mouse_press(&event, &mut self.ctx);
    }

    pub fn mouse_up(&mut self, event: &MouseUnpress) {
        self.ctx.pressed_mb.insert(event.button, false);
        self.root_view.mouse_unpress(&event, &mut self.ctx);
    }
}
