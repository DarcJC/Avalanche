pub mod window;

use ash::vk;
use bevy_ecs::prelude::Entity;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub struct ExtractedWindow<T: HasWindowHandle + HasDisplayHandle> {
    pub entity: Entity,
    pub window: T,
    pub physical_width: u32,
    pub physical_height: u32,
    pub present_mode: vk::PresentModeKHR,
}

