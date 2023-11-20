pub mod core;
mod ash_window;

use crate::core::event_loop::EventLoopManager;
use crate::core::renderer_trait::Renderer;
use crate::core::renderer_vulkan::VulkanRenderer;
use crate::core::window_manager::WindowManager;

fn main() {
    let mut event_loop_manager = EventLoopManager::new();
    let mut renderer = VulkanRenderer::new();
    renderer.list_physical_devices();

    let mut window_manager = WindowManager::new(renderer);
    window_manager.create_window(&mut event_loop_manager, "QwQ", 800, 600);
    event_loop_manager.run(|event, target_window| {
    });
}
