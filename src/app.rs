use crate::{event::UserEvent, window::GlutinWindowContext};
use std::sync::Arc;

pub struct GlowApp {
    proxy: winit::event_loop::EventLoopProxy<UserEvent>,
    gl_window: Option<GlutinWindowContext>,
    gl: Option<Arc<glow::Context>>,
    egui_glow: Option<egui_glow::EguiGlow>,
    repaint_delay: std::time::Duration,
    clear_color: [f32; 3],
    window_hidden: bool,
    update_ui: Box<dyn Fn(&egui::Context) + Send + Sync + 'static>,
}

impl GlowApp {
    pub fn new(
        proxy: winit::event_loop::EventLoopProxy<UserEvent>,
        update_ui: Box<dyn Fn(&egui::Context) + Send + Sync>,
    ) -> Self {
        Self {
            proxy,
            gl_window: None,
            gl: None,
            egui_glow: None,
            repaint_delay: std::time::Duration::MAX,
            clear_color: [0.1, 0.1, 0.1],
            window_hidden: false,
            update_ui,
        }
    }
}

fn create_display(
    event_loop: &winit::event_loop::ActiveEventLoop,
) -> (GlutinWindowContext, glow::Context) {
    let glutin_window_context = unsafe { GlutinWindowContext::new(event_loop) };
    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            let s = std::ffi::CString::new(s)
                .expect("failed to construct C string from string for gl proc address");

            glutin_window_context.get_proc_address(&s)
        })
    };

    (glutin_window_context, gl)
}

impl winit::application::ApplicationHandler<UserEvent> for GlowApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let (gl_window, gl) = create_display(event_loop);
        let gl = std::sync::Arc::new(gl);
        gl_window.window().set_visible(true);

        let egui_glow = egui_glow::EguiGlow::new(event_loop, gl.clone(), None, None, true);

        let event_loop_proxy = egui::mutex::Mutex::new(self.proxy.clone());
        egui_glow
            .egui_ctx
            .set_request_repaint_callback(move |info| {
                event_loop_proxy
                    .lock()
                    .send_event(UserEvent::Redraw(info.delay))
                    .expect("Cannot send event");
            });
        self.gl_window = Some(gl_window);
        self.gl = Some(gl);
        self.egui_glow = Some(egui_glow);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let mut redraw = || {
            let mut quit = false;

            let gl_window = self.gl_window.as_mut().unwrap();
            self.egui_glow
                .as_mut()
                .unwrap()
                .run(gl_window.window(), self.update_ui.as_mut());

            if quit {
                event_loop.exit();
            } else {
                event_loop.set_control_flow(if self.repaint_delay.is_zero() {
                    self.gl_window.as_mut().unwrap().window().request_redraw();
                    winit::event_loop::ControlFlow::Poll
                } else if let Some(repaint_after_instant) =
                    std::time::Instant::now().checked_add(self.repaint_delay)
                {
                    winit::event_loop::ControlFlow::WaitUntil(repaint_after_instant)
                } else {
                    winit::event_loop::ControlFlow::Wait
                });
            }

            {
                unsafe {
                    use glow::HasContext as _;
                    self.gl.as_mut().unwrap().clear_color(
                        self.clear_color[0],
                        self.clear_color[1],
                        self.clear_color[2],
                        1.0,
                    );
                    self.gl.as_mut().unwrap().clear(glow::COLOR_BUFFER_BIT);
                }

                // draw things behind egui here
                // 在egui后面绘制内容

                self.egui_glow
                    .as_mut()
                    .unwrap()
                    .paint(self.gl_window.as_mut().unwrap().window());

                // draw things on top of egui here
                // 在egui上面绘制内容

                // Only make the window visible at the end of rendering if it's not supposed to be hidden
                // 只有在不应该隐藏窗口的情况下，才在渲染结束时使窗口可见
                if !self.window_hidden {
                    self.gl_window.as_mut().unwrap().swap_buffers().unwrap();
                } else {
                    // Even when hidden, we still need to swap buffers
                    // 即使隐藏，我们仍需要交换缓冲区
                    self.gl_window.as_mut().unwrap().swap_buffers().unwrap();
                    // But we ensure the window stays hidden
                    // 但是我们确保窗口保持隐藏状态
                    self.gl_window.as_mut().unwrap().window().set_visible(false);
                }
            }
        };

        use winit::event::WindowEvent;
        if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
            event_loop.exit();
            return;
        }

        if matches!(event, WindowEvent::RedrawRequested) {
            redraw();
            return;
        }

        if let winit::event::WindowEvent::Resized(physical_size) = &event {
            self.gl_window.as_mut().unwrap().resize(*physical_size);
        }

        let event_response = self
            .egui_glow
            .as_mut()
            .unwrap()
            .on_window_event(self.gl_window.as_mut().unwrap().window(), &event);

        if event_response.repaint {
            self.gl_window.as_mut().unwrap().window().request_redraw();
        }
    }

    // !NOTICE: user event handler
    // !注意: 用户事件处理器
    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Redraw(delay) => self.repaint_delay = delay,
            UserEvent::ShowWindow => {
                self.window_hidden = false;
                if let Some(ref gl_window) = self.gl_window {
                    gl_window.window().set_visible(true);
                    gl_window.window().request_redraw();
                }
            }
            UserEvent::HideWindow => {
                self.window_hidden = true;
                if let Some(ref gl_window) = self.gl_window {
                    gl_window.window().set_visible(false);
                }
            }
        }
    }

    fn new_events(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        if let winit::event::StartCause::ResumeTimeReached { .. } = &cause {
            self.gl_window.as_mut().unwrap().window().request_redraw();
        }
    }

    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.egui_glow.as_mut().unwrap().destroy();
    }
}
