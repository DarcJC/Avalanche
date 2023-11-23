#![feature(associated_type_defaults)]
#![feature(slice_flatten)]

use std::ops::DerefMut;
use ash::vk;
use tobj::LoadOptions;
use avalanche::core::event_loop::EventLoopManager;
use avalanche::core::renderer_trait::{RayTracingRenderer, Renderer};
use avalanche::core::renderer_types::BLASBuildData;
use avalanche::core::renderer_vulkan::VulkanBuffer;
use avalanche::core::scene::{MeshBuffers, TObjMeshWrapper};
use avalanche::core::window_manager::{get_window_manager, WindowManagerTrait};

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let mut event_loop_manager = EventLoopManager::new();
    {
        let mut window_manager = get_window_manager().await;
        window_manager.create_window(&mut event_loop_manager, "QwQ", 800, 600).await;
        window_manager.renderer.lock().await.initialize();

        let mut renderer = window_manager.renderer.lock().await;
        renderer.list_physical_devices();
    }

    test_ray_tracing();

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

fn test_ray_tracing() {
    let mut build_input = BLASBuildData::default();

    let (model, _mat) = tobj::load_obj("C:/Users/DarcJC/Desktop/cube.obj", &LoadOptions::default()).expect("Failed to load test model.");

    model.iter().for_each(|model| build_input.geometries.push(Box::new(TObjMeshWrapper::<VulkanBuffer>::from(model.mesh.clone()))));

    let mut model = TObjMeshWrapper::<VulkanBuffer>::from(model.first().unwrap().mesh.clone());
    let vb = model.get_or_create_vertex_buffer();
    println!("{:?}", vb);

    // renderer.build_bottom_level_acceleration_structure(&build_input);
}
