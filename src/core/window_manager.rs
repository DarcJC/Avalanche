use crate::core::event_loop::EventLoopManager;
use crate::core::renderer_trait::Renderer;

pub struct WindowManager<R: Renderer> {
    renderer: R,
    windows: Vec<winit::window::Window>,
}

impl<R: Renderer> WindowManager<R> {
    pub fn new(renderer: R) -> Self {
        WindowManager {
            renderer,
            windows: Vec::new(),
        }
    }

    pub fn create_window(&mut self, event_loop: &mut EventLoopManager, title: &str, width: u32, height: u32) {
        let window = self.renderer.create_window(event_loop, title, width, height);
        self.windows.push(window);
    }

    pub fn request_to_redraw_window(&mut self, window_id: winit::window::WindowId) {
        if let Some(window) = self.windows.iter().find(|window| window.id() == window_id) {
            window.request_redraw();
        }
    }

    pub fn request_redraw_all_windows(&mut self) {
        self.windows.iter().for_each(|window| window.request_redraw());
    }
}
