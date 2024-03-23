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

pub struct UIApp {
    handle: WindowHandle,
    size: Size,
    last_time: Instant,
    root_view: ViewId,
    ctx: Context,
    title: String,
    background_color: Color,
    event_queue: Vec<EventJob>,
}

impl UIApp {
    pub fn new(root_view: ViewId, ctx: Context, window_properties: WindowProperties) -> Self {
        Self {
            size: Size::ZERO,
            handle: Default::default(),
            last_time: time::Instant::now(),
            root_view,
            ctx,
            title: window_properties.title,
            background_color: window_properties.backdround_color,
            event_queue: vec![],
        }
    }

    fn add_update_event_job(&mut self) {
        let event = Event::Update;
        let views = self
            .ctx
            .arena
            .keys()
            .map(|key| *key)
            .collect::<Vec<ViewId>>();
        self.event_queue.push(EventJob { event, views });
    }

    /// Returns true if view processed the event
    fn pass_event_to_view(&mut self, id: ViewId, event: &Event) -> bool {
        self.ctx
            .map_view(id, &mut |view, ctx| view.process_event(event, ctx))
    }

    fn process_event_job(&mut self, job: &EventJob) {
        match &job.event {
            // We must save the views that got mouse press event to give them
            // mouse unpress event. That is usable in text editors to prevent
            // scroll text with select
            Event::MousePress(button) => {
                let mut processed = vec![];
                for id in &job.views {
                    if self.pass_event_to_view(*id, &job.event) {
                        processed.push(*id);
                    }
                }
                self.ctx.pressed_mb.insert(*button, (true, processed));
            }
            event => {
                for id in &job.views {
                    self.pass_event_to_view(*id, event);
                }
            }
        }
    }

    fn process_event_queue(&mut self) {
        let event_queue = std::mem::take(&mut self.event_queue);
        for job in &event_queue {
            self.process_event_job(job);
        }
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
        self.ctx.map_view(self.root_view, &mut |view, ctx| {
            view.draw(DrawContext {
                id: self.root_view,
                drawer: piet,
                size: self.size,
                ctx,
            })
        });
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

    fn after_draw(&mut self, _piet: &mut Piet) {
        self.handle.request_anim_frame();
    }
}

impl WinHandler for UIApp {
    fn connect(&mut self, handle: &WindowHandle) {
        self.handle = handle.clone();
    }

    fn prepare_paint(&mut self) {
        self.handle.invalidate();
    }

    fn paint(&mut self, piet: &mut Piet, _: &Region) {
        self.add_update_event_job();

        self.process_event_queue();

        // TODO
        self.update_window_data(vec![]);

        self.clear_surface(piet);

        self.draw_view(piet);

        self.draw_debug_data(piet);

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
        self.ctx.pressed_mb.remove(&event.button);
        let mut views = vec![];
        for (key, layout) in self.ctx.layouts.iter() {
            if layout.intersects(event.pos) {
                views.push(*key);
            }
        }
        self.event_queue.push(EventJob {
            event: Event::MousePress(event.button),
            views,
        })
    }

    fn mouse_up(&mut self, event: &MouseEvent) {
        if let Some((_, views)) = self.ctx.pressed_mb.remove(&event.button) {
            self.event_queue.push(EventJob {
                event: Event::MouseUnpress(event.button),
                views,
            });
        }
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
