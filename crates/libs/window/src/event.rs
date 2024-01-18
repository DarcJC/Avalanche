use bevy_ecs::prelude::Event;
use winit::event::WindowEvent;
use winit::window::WindowId;

#[derive(Event)]
pub struct WinitWindowEvent {
    pub window_event: WindowEvent,
    pub window_id: WindowId,
}

#[derive(Event)]
pub struct WindowClosedEvent {
    pub window_id: WindowId,
}

/// ## Window resized event
///
/// Delegated to application because we don't have rendering context to perform operation
#[derive(Event)]
pub struct WindowResizedEvent {
    pub window_id: WindowId,
    /// width, height
    pub new_size: (u32, u32),
}

#[derive(Event)]
pub struct WindowEventLoopClearedEvent();
