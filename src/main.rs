#![feature(associated_type_defaults)]
#![feature(slice_flatten)]

use tobj::LoadOptions;
use anyhow::Result;
use avalanche::async_get_renderer_as_var;
use avalanche::core::event_loop::EventLoopManager;
use avalanche::core::renderer_trait::{MeshBuffers, RayTracingRenderer, Renderer};
use avalanche::core::renderer_types::BLASBuildData;
use avalanche::core::renderer_vulkan::VulkanBuffer;
use avalanche::core::scene::{TObjMeshWrapper};
use avalanche::core::window_manager::{get_window_manager, WindowManagerTrait};

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let mut event_loop_manager = EventLoopManager::new();
    {
        let mut window_manager = get_window_manager().await;
        window_manager.create_window(&mut event_loop_manager, "QwQ", 800, 600).await;
        window_manager.renderer.lock().await.initialize();

        let renderer = window_manager.renderer.lock().await;
        renderer.list_physical_devices();
    }

    test_ray_tracing().await.unwrap();

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

async fn test_ray_tracing() -> Result<()> {
    let mut build_input = BLASBuildData::default();

    let (model, _mat) = tobj::load_obj("C:/Users/DarcJC/Desktop/cube.obj", &LoadOptions::default()).expect("Failed to load test model.");

    let mut model = TObjMeshWrapper::<VulkanBuffer>::from(model.first().unwrap().mesh.clone());
    async_get_renderer_as_var!(window_manager, renderer);
    let geometry = renderer.mesh_buffers_to_geometry(&mut model)?;
    build_input.geometries.push(geometry);

    // renderer.build_bottom_level_acceleration_structure(&build_input);
    Ok(())
}
