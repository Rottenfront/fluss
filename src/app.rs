use super::*;
use std::time::Instant;
use winit::window::Window;

pub struct UIApp<V: View + 'static> {
    window: Window,
    renderer: Renderer,
    root_view: V,
    ctx: Context,
    title: String,
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
        }
    }

    fn update_window_data(&mut self) {
        let actions = std::mem::take(&mut self.ctx.actions);
        for action in actions {
            match action {
                Action::SetTitle(title) => {
                    self.title = title;
                    self.window.set_title(&self.title);
                }
                Action::SetCursor(cursor) => self.window.set_cursor_icon(cursor),
                Action::Quit => {}
            }
        }

        if self.ctx.need_redraw {
            self.window.request_redraw();
        }
    }

    pub fn request_redraw(&mut self) {
        self.window.request_redraw();
    }

    pub fn resize(&mut self) {
        self.renderer
            .resize(self.window.inner_size(), self.window.scale_factor());
    }

    pub fn update(&mut self) {
        self.root_view.update(&mut self.ctx);

        self.update_window_data();
    }

    fn draw_view(&mut self) {
        self.renderer.clear();

        let size = self.renderer.size();

        self.root_view.draw(DrawContext {
            drawer: &mut self.renderer,
            size,
            ctx: &mut self.ctx,
        })
    }

    fn after_draw(&mut self) {
        self.renderer.present();
    }

    pub fn paint(&mut self) {
        self.draw_view();

        self.after_draw();
    }

    pub fn key_down(&mut self, event: KeyboardEvent) {}

    pub fn key_up(&mut self, event: KeyboardEvent) {}

    pub fn scroll(&mut self, event: ScrollEvent) {
        // println!("mouse_wheel {event:?}");
    }

    pub fn cursor_move(&mut self, event: MouseMoveEvent) {
        self.ctx.pointer = event.pos.to_vec2();
    }

    pub fn mouse_press(&mut self, event: MousePress) {
        self.ctx.pressed_mb.insert(event.button, true);
        self.root_view.mouse_press(&event, &mut self.ctx);
    }

    pub fn mouse_unpress(&mut self, event: MouseUnpress) {
        self.ctx.pressed_mb.insert(event.button, false);
        self.root_view.mouse_unpress(&event, &mut self.ctx);
    }

    pub fn run(&mut self, event_loop: EventLoop<()>) {
        tracing_subscriber::fmt::init();

        let mut previous_frame_start = Instant::now();
        let mut cursor_pos = Point::new(0.0, 0.0);

        // Run event loop
        event_loop
            .run(move |event, window_target| {
                let frame_start = Instant::now();

                match event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::CursorMoved { position, .. } => {
                            // cursor_position = Some(position);
                            cursor_pos = Point::new(position.x, position.y);
                            self.cursor_move(MouseMoveEvent { pos: cursor_pos });
                        }
                        WindowEvent::MouseInput { state, button, .. } => {
                            if state.is_pressed() {
                                self.mouse_press(MousePress {
                                    button,
                                    pos: cursor_pos,
                                });
                            } else {
                                self.mouse_unpress(MouseUnpress {
                                    button,
                                    pos: cursor_pos,
                                });
                            }
                        }
                        WindowEvent::ModifiersChanged(new_modifiers) => {
                            // modifiers = new_modifiers.state();
                        }
                        WindowEvent::Resized(_) => {
                            self.resize();
                            self.request_redraw();
                        }
                        WindowEvent::CloseRequested => {
                            window_target.exit();
                        }
                        WindowEvent::RedrawRequested => {
                            self.paint();
                        }
                        _ => {}
                    },
                    _ => {}
                }

                let expected_frame_length_seconds = 1.0 / 60.0;
                let frame_duration = Duration::from_secs_f32(expected_frame_length_seconds);

                if frame_start - previous_frame_start > frame_duration {
                    self.update();
                    previous_frame_start = frame_start;
                }

                window_target.set_control_flow(ControlFlow::WaitUntil(
                    previous_frame_start + frame_duration,
                ))
            })
            .unwrap();
    }
}
