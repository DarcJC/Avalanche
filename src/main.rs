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
    window_manager.borrow_renderer_mut().initialize();

    event_loop_manager.run(move |event, target_window| {
        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                window_id,
            } => {
                target_window.exit();
            },
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::RedrawRequested,
                window_id,
            } => {
                // call graphics api to draw
            },
            winit::event::Event::AboutToWait => {
                window_manager.request_redraw_all_windows();
            },
            _ => {},
        };
    });
}
