use crate::core::event_loop::EventLoopManager;
use crate::core::renderer_trait::{GraphicsAbstract, Renderer};

pub struct WindowManager<R: Renderer> {
    renderer: R,
    windows: Vec<winit::window::Window>,
}

pub trait WindowManagerTrait {
    type Renderer: GraphicsAbstract;
    fn new(renderer: Self::Renderer) -> Self;
    fn borrow_renderer(&self) -> &Self::Renderer;
    fn borrow_renderer_mut(&mut self) -> &mut Self::Renderer;
    fn create_window(&mut self, event_loop: &mut EventLoopManager, title: &str, width: u32, height: u32);
    fn request_to_redraw_window(&mut self, window_id: winit::window::WindowId);
    fn request_redraw_all_windows(&mut self);
}

impl<R: GraphicsAbstract> WindowManagerTrait for WindowManager<R> {
    type Renderer = R;

    fn new(renderer: R) -> Self {
        WindowManager {
            renderer,
            windows: Vec::new(),
        }
    }

    fn borrow_renderer(&self) -> &R {
        &self.renderer
    }

    fn borrow_renderer_mut(&mut self) -> &mut R {
        &mut self.renderer
    }

    fn create_window(&mut self, event_loop: &mut EventLoopManager, title: &str, width: u32, height: u32) {
        let window = self.renderer.create_window(event_loop, title, width, height);
        self.windows.push(window);
    }

    fn request_to_redraw_window(&mut self, window_id: winit::window::WindowId) {
        if let Some(window) = self.windows.iter().find(|window| window.id() == window_id) {
            window.request_redraw();
        }
    }

    fn request_redraw_all_windows(&mut self) {
        self.windows.iter().for_each(|window| window.request_redraw());
    }
}

pub struct WindowManagerWrapper {
}
