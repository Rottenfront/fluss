use super::*;
use shell::{
    kurbo::Size,
    piet::{Color, FontFamily, Piet, RenderContext, Text, TextLayoutBuilder},
    Application, Cursor, FileDialogToken, FileInfo, KeyEvent, MouseEvent, Region, TimerToken,
    WinHandler, WindowHandle,
};
use time::Instant;

struct EventJob {
    event: Event,
    views: Vec<ViewId>,
}

pub struct UIApp<V: View + 'static> {
    handle: WindowHandle,
    size: Size,
    last_time: Instant,
    root_view: V,
    ctx: Context,
    title: String,
    background_color: Color,
}

impl<V: View + 'static> UIApp<V> {
    pub fn new(root_view: V, window_properties: WindowProperties) -> Self {
        Self {
            size: Size::ZERO,
            handle: Default::default(),
            last_time: time::Instant::now(),
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
                Action::SetCursor(cursor) => self.handle.set_cursor(&cursor),
                Action::SetBackgroundColor(color) => self.background_color = color,
                Action::Quit => self.handle.close(),
            }
        }
    }

    fn clear_surface(&mut self, piet: &mut Piet) {
        let rect = self.size.to_rect();
        piet.clear(rect, self.background_color);
    }

    fn draw_view(&mut self, piet: &mut Piet) {
        self.root_view.draw(DrawContext {
            drawer: piet,
            size: self.size,
            ctx: &mut self.ctx,
        })
    }

    fn draw_debug_data(&mut self, piet: &mut Piet, before_draw_time: Instant) {
        let now = Instant::now();
        let full_frame = format!("{}ms", (now - self.last_time).whole_milliseconds());
        let just_draw = format!("{}micros", (now - before_draw_time).whole_microseconds());

        self.last_time = now;

        let layout = piet
            .text()
            .new_text_layout(full_frame)
            .font(FontFamily::MONOSPACE, 14.0)
            .text_color(Color::WHITE)
            .build()
            .unwrap();

        piet.draw_text(&layout, (0.0, 0.0));
        let layout = piet
            .text()
            .new_text_layout(just_draw)
            .font(FontFamily::MONOSPACE, 14.0)
            .text_color(Color::WHITE)
            .build()
            .unwrap();

        piet.draw_text(&layout, (100.0, 0.0));
    }

    fn after_draw(&mut self, _piet: &mut Piet) {
        self.handle.request_anim_frame();
    }
}

impl<V: View + 'static> WinHandler for UIApp<V> {
    fn connect(&mut self, handle: &WindowHandle) {
        self.handle = handle.clone();
    }

    fn prepare_paint(&mut self) {
        self.handle.invalidate();
    }

    fn paint(&mut self, piet: &mut Piet, _: &Region) {
        let before_draw = Instant::now();

        self.update_window_data();

        self.clear_surface(piet);

        self.draw_view(piet);

        self.draw_debug_data(piet, before_draw);

        self.after_draw(piet);
    }

    fn command(&mut self, id: u32) {
        match id {
            0x100 => self.handle.close(),
            _ => println!("unexpected id {id}"),
        }
    }

    fn open_file(&mut self, _token: FileDialogToken, file_info: Option<FileInfo>) {
        println!("open file result: {file_info:?}");
    }

    fn save_as(&mut self, _token: FileDialogToken, file: Option<FileInfo>) {
        println!("save file result: {file:?}");
    }

    fn key_down(&mut self, event: KeyEvent) -> bool {
        println!("keydown: {event:?}");
        false
    }

    fn key_up(&mut self, event: KeyEvent) {
        println!("keyup: {event:?}");
    }

    fn wheel(&mut self, event: &MouseEvent) {
        println!("mouse_wheel {event:?}");
    }

    fn mouse_move(&mut self, event: &MouseEvent) {
        self.ctx.pointer = event.pos.to_vec2();
    }

    fn mouse_down(&mut self, event: &MouseEvent) {
        self.ctx.pressed_mb.insert(event.button, true);
        self.root_view.process_event(
            &Event::MousePress {
                button: event.button,
                pos: event.pos,
            },
            &mut self.ctx,
        );
    }

    fn mouse_up(&mut self, event: &MouseEvent) {
        self.ctx.pressed_mb.insert(event.button, false);
        self.root_view.process_event(
            &Event::MouseUnpress {
                button: event.button,
                pos: event.pos,
            },
            &mut self.ctx,
        );
    }

    fn timer(&mut self, id: TimerToken) {
        println!("timer fired: {id:?}");
    }

    fn size(&mut self, size: Size) {
        self.size = size;
    }

    fn got_focus(&mut self) {
        self.handle.set_cursor(&Cursor::Arrow);
        println!("Got focus");
    }

    fn lost_focus(&mut self) {
        println!("Lost focus");
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
