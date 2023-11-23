#![feature(associated_type_defaults)]
#![feature(slice_flatten)]

use std::ops::DerefMut;
use ash::vk;
use tobj::LoadOptions;
use avalanche::core::event_loop::EventLoopManager;
use avalanche::core::renderer_trait::{RayTracingRenderer, Renderer};
use avalanche::core::renderer_types::BLASBuildData;
use avalanche::core::renderer_vulkan::VulkanBuffer;
use avalanche::core::scene::TObjMeshWrapper;
use avalanche::core::window_manager::{get_window_manager, WindowManagerTrait};

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let mut event_loop_manager = EventLoopManager::new();
    {
        let mut window_manager = get_window_manager().await;
        window_manager.create_window(&mut event_loop_manager, "QwQ", 800, 600).await;
        window_manager.renderer.lock().await.initialize();

        let mut renderer = window_manager.renderer.lock().await;
        test_ray_tracing(renderer.deref_mut());
        renderer.list_physical_devices();
    }

    event_loop_manager.run(|event, target_window| {
        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                window_id: _window_id,
            } => {
                target_window.exit();
            },
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::RedrawRequested,
                window_id: _window_id,
            } => {
                // call graphics api to draw
            },
            winit::event::Event::AboutToWait => {
                let mut window_manager = async_std::task::block_on(get_window_manager());
                window_manager.request_redraw_all_windows();
            },
            _ => {},
        };
    });

    Ok(())
}

fn test_ray_tracing(renderer: &mut (impl RayTracingRenderer + Renderer)) {
    let mut build_input = BLASBuildData::default();

    let (model, _mat) = tobj::load_obj("C:/Users/DarcJC/Desktop/cube.obj", &LoadOptions::default()).expect("Failed to load test model.");

    model.iter().for_each(|model| build_input.geometries.push(Box::new(TObjMeshWrapper::<VulkanBuffer>::from(model.mesh.clone()))));

    let mut buffer = VulkanBuffer::default();
    buffer.create_info.s_type = vk::StructureType::BUFFER_CREATE_INFO;
    buffer.create_info.size = 32;
    buffer.create_info.usage = vk::BufferUsageFlags::VERTEX_BUFFER;
    buffer.create_info.sharing_mode = vk::SharingMode::EXCLUSIVE;
    buffer.create_info.flags = vk::BufferCreateFlags::empty();

    renderer.create_buffer_resource(&mut buffer);

    renderer.build_bottom_level_acceleration_structure(&build_input);
}
