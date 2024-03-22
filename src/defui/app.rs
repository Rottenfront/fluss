use super::*;
use shell::{
    kurbo::Size,
    piet::{Color, FontFamily, Piet, RenderContext, Text, TextLayoutBuilder},
    Application, Cursor, FileDialogToken, FileInfo, KeyEvent, MouseEvent, Region, TimerToken,
    WinHandler, WindowHandle,
};
use time::Instant;

pub struct UIApp<V: View + 'static> {
    handle: WindowHandle,
    size: Size,
    last_time: Instant,
    view: V,
    ctx: Context,
    title: String,
    background_color: Color,
}

impl<V: View + 'static> UIApp<V> {
    pub fn new(view: V, ctx: Context, window_properties: WindowProperties) -> Self {
        Self {
            size: Size::ZERO,
            handle: Default::default(),
            last_time: time::Instant::now(),
            view,
            ctx,
            title: window_properties.title,
            background_color: window_properties.backdround_color,
        }
    }

    fn update_view(&mut self, piet: &mut Piet) {
        self.view.process_event(&Event::Update, &mut self.ctx, piet);
    }

    fn update_window_data(&mut self, actions: Vec<Action>) {
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
        self.view.draw(ViewId(0), piet, self.size, &mut self.ctx);
    }

    fn draw_debug_data(&mut self, piet: &mut Piet) {
        let now = Instant::now();
        let msg = format!("{}ms", (now - self.last_time).whole_milliseconds());

        self.last_time = now;
        let layout = piet
            .text()
            .new_text_layout(msg)
            .font(FontFamily::MONOSPACE, 14.0)
            .text_color(Color::WHITE)
            .build()
            .unwrap();

        piet.draw_text(&layout, (0.0, 0.0));
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
        self.update_view(piet);

        self.update_window_data(vec![]);

        self.clear_surface(piet);

        self.draw_view(piet);

        self.draw_debug_data(piet);

        self.handle.request_anim_frame();
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
        self.handle.set_cursor(&Cursor::Arrow);
        println!("mouse_move {event:?}");
    }

    fn mouse_down(&mut self, event: &MouseEvent) {
        for (key, layout) in self.ctx.layouts.iter() {
            if layout.intersects(event.pos) {
                println!("{key:?} {layout:?}");
            }
        }
    }

    fn mouse_up(&mut self, event: &MouseEvent) {
        println!("mouse_up {event:?}");
    }

    fn timer(&mut self, id: TimerToken) {
        println!("timer fired: {id:?}");
    }

    fn size(&mut self, size: Size) {
        self.size = size;
    }

    fn got_focus(&mut self) {
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
