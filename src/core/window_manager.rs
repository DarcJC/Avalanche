use async_std::sync::{Mutex, MutexGuard};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use crate::core::event_loop::EventLoopManager;
use crate::core::renderer_trait::{GraphicsAbstract, Renderer};
use crate::core::renderer_vulkan::VulkanRenderer;

pub struct WindowManager<R: Renderer> {
    pub renderer: Mutex<R>,
    windows: Vec<winit::window::Window>,
}

#[async_trait]
pub trait WindowManagerTrait {
    type Renderer: GraphicsAbstract;
    fn new(renderer: Self::Renderer) -> Self;
    async fn create_window(&mut self, event_loop: &mut EventLoopManager, title: &str, width: u32, height: u32);
    fn request_to_redraw_window(&mut self, window_id: winit::window::WindowId);
    fn request_redraw_all_windows(&mut self);
}

#[async_trait]
impl<R: GraphicsAbstract + Send> WindowManagerTrait for WindowManager<R> {
    type Renderer = R;

    fn new(renderer: R) -> Self {
        WindowManager {
            renderer: Mutex::new(renderer),
            windows: Vec::new(),
        }
    }

    async fn create_window(&mut self, event_loop: &mut EventLoopManager, title: &str, width: u32, height: u32) {
        let window = self.renderer.lock().await.create_window(event_loop, title, width, height);
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

unsafe impl<R: Renderer> Sync for WindowManager<R> {}

type RendererType = VulkanRenderer;

static mut WINDOW_MANAGER: Lazy<Mutex<WindowManager<RendererType>>> = Lazy::new(|| {
    let renderer = VulkanRenderer::new();
    Mutex::new(WindowManager::new(renderer))
});

pub async fn get_window_manager() -> MutexGuard<'static, WindowManager<RendererType>> {
    unsafe { WINDOW_MANAGER.lock().await }
}
