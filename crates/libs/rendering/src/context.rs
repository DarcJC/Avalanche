use std::sync::Arc;
use bevy_ecs::prelude::Resource;
use avalanche_hlvk::{CommandBuffer, CommandPool, Context};

#[derive(Resource)]
pub struct RenderingContext {
    pub context: Arc<Context>,
    pub command_pool: Arc<CommandPool>,
    pub swapchain_command_buffer: Arc<Vec<CommandBuffer>>,
}

impl Clone for RenderingContext {
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
            command_pool: self.command_pool.clone(),
            swapchain_command_buffer: self.swapchain_command_buffer.clone(),
        }
    }
}
