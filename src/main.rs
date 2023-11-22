#![feature(associated_type_defaults)]
#![feature(slice_flatten)]

pub mod core;
mod ash_window;

use ash::vk;
use tobj::LoadOptions;
use crate::core::event_loop::EventLoopManager;
use crate::core::renderer_trait::{RayTracingRenderer, Renderer};
use crate::core::renderer_types::BLASBuildData;
use crate::core::renderer_vulkan::{VulkanBuffer, VulkanRenderer};
use crate::core::scene::TObjMeshWrapper;
use crate::core::window_manager::{WINDOW_MANAGER, WindowManager, WindowManagerTrait};

fn main() {
    let mut event_loop_manager = EventLoopManager::new();
    let mut window_manager = WINDOW_MANAGER.lock().unwrap();
    window_manager.create_window(&mut event_loop_manager, "QwQ", 800, 600);
    window_manager.borrow_renderer_mut().initialize();

    let renderer = window_manager.borrow_renderer_mut();
    test_ray_tracing(renderer);
    renderer.list_physical_devices();

    event_loop_manager.run(move |event, target_window| {
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
                window_manager.request_redraw_all_windows();
            },
            _ => {},
        };
    });
}

fn test_ray_tracing(renderer: &mut (impl RayTracingRenderer + Renderer)) {
    let mut build_input = BLASBuildData::default();

    let (model, mat) = tobj::load_obj("C:/Users/DarcJC/Desktop/cube.obj", &LoadOptions::default()).expect("Failed to load test model.");

    model.iter().for_each(|model| build_input.geometries.push(Box::new(TObjMeshWrapper::from(model.mesh.clone()))));

    let mut buffer = VulkanBuffer::default();
    buffer.create_info.s_type = vk::StructureType::BUFFER_CREATE_INFO;
    buffer.create_info.size = 32;
    buffer.create_info.usage = vk::BufferUsageFlags::VERTEX_BUFFER;
    buffer.create_info.sharing_mode = vk::SharingMode::EXCLUSIVE;
    buffer.create_info.flags = vk::BufferCreateFlags::empty();

    renderer.create_buffer_resource(&mut buffer);

    renderer.build_bottom_level_acceleration_structure(&build_input);
}
