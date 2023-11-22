use std::any::Any;
use std::ptr::addr_of_mut;
use crate::core::event_loop::EventLoopManager;
use crate::core::renderer_types::{BLASBuildData, GraphicsAPIType};

pub trait Renderer {
    fn new() -> Self where Self: Sized;
    fn create_window(&mut self, event_loop: &mut EventLoopManager, title: &str, width: u32, height: u32) -> winit::window::Window;
    fn list_physical_devices(&self);
    fn initialize(&mut self);
    fn support_ray_tracing(&self) -> bool;

    /// Call to perform real buffer allocation on GPU.
    fn create_buffer_resource(&mut self, buffer: &mut impl Buffer);
}

pub trait RayTracingRenderer {
    fn build_bottom_level_acceleration_structure(&mut self, inputs: &BLASBuildData);
}

pub trait GraphicAPIBounds {
    fn get_graphics_api() -> GraphicsAPIType where Self: Sized;
}

pub trait Buffer where Self: GraphicAPIBounds {
    fn get_buffer_name<'a>() -> &'a str where Self: Sized;
    fn release(&mut self);
}

pub fn buffer_cast<'a, TargetType: Buffer + 'a, U: Buffer + 'a>(buffer: &mut U) -> Option<&mut TargetType> {
    if TargetType::get_graphics_api() == U::get_graphics_api() {
        Some(unsafe { std::mem::transmute(buffer) })
    } else {
        None
    }
}
