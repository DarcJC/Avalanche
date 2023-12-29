use std::sync::Arc;
use bevy_ecs::prelude::Event;
use avalanche_hlvk::{AcquiredImage, Fence, Semaphore};
use avalanche_window::WindowId;

#[derive(Event)]
pub struct BeginRenderWindowViewEvent {
    /// Window Id
    pub window_id: WindowId,
    /// Semaphore to signal present image
    pub frame_finish_semaphore: Arc<Semaphore>,
    /// Semaphore to signal the next image is ready.
    /// We can start to render this frame after it is signaled.
    pub image_acquire_semaphore: Arc<Semaphore>,
    /// The window swapchain image we will render on
    pub window_image: AcquiredImage,
    /// Fence to wait on swapchain present
    pub working_fence: Arc<Fence>,
}
