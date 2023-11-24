use async_std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use crate::core::event_loop::EventLoopManager;
use crate::core::renderer_trait::{GraphicsAbstract, Renderer};
use crate::core::renderer_vulkan::VulkanRenderer;

pub struct WindowManager<R: Renderer> {
    pub renderer: RwLock<R>,
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
impl<R: GraphicsAbstract + Send + Sync> WindowManagerTrait for WindowManager<R> {
    type Renderer = R;

    fn new(renderer: R) -> Self {
        WindowManager {
            renderer: RwLock::new(renderer),
            windows: Vec::new(),
        }
    }

    async fn create_window(&mut self, event_loop: &mut EventLoopManager, title: &str, width: u32, height: u32) {
        let window = self.renderer.write().await.create_window(event_loop, title, width, height);
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

pub type RendererType = VulkanRenderer;

static mut WINDOW_MANAGER: Lazy<RwLock<WindowManager<RendererType>>> = Lazy::new(|| {
    let renderer = VulkanRenderer::new();
    RwLock::new(WindowManager::new(renderer))
});

pub async fn get_window_manager() -> RwLockReadGuard<'static, WindowManager<RendererType>> {
    unsafe { WINDOW_MANAGER.read().await }
}

pub async fn get_window_manager_mut() -> RwLockWriteGuard<'static, WindowManager<RendererType>> {
    unsafe { WINDOW_MANAGER.write().await }
}

#[macro_export]
macro_rules! async_get_renderer_as_var {
    ($window_manager_name:ident, $var_name:ident) => {
        let $window_manager_name = get_window_manager().await;
        let $var_name = $window_manager_name.renderer.read().await;
    };
}

#[macro_export]
macro_rules! async_get_renderer_as_var_mut {
    ($window_manager_name:ident, $var_name:ident) => {
        let $window_manager_name = get_window_manager().await;
        let mut $var_name = $window_manager_name.renderer.write().await;
    };
}

#[macro_export]
macro_rules! get_renderer_as_var {
    ($window_manager_name:ident, $var_name:ident) => {
        let $window_manager_name = async_std::task::block_on(get_window_manager());
        let $var_name = async_std::task::block_on($window_manager_name.renderer.read());
    };
}

#[macro_export]
macro_rules! get_renderer_as_var_mut {
    ($window_manager_name:ident, $var_name:ident) => {
        let $window_manager_name = async_std::task::block_on(get_window_manager());
        let mut $var_name = async_std::task::block_on($window_manager_name.renderer.write());
    };
}
