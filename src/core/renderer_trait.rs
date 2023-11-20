use crate::core::event_loop::EventLoopManager;

pub trait Renderer {
    fn new() -> Self where Self: Sized;
    fn create_window(&mut self, event_loop: &mut EventLoopManager, title: &str, width: u32, height: u32) -> winit::window::Window;
    fn list_physical_devices(&self);
    fn initialize(&mut self);
}
