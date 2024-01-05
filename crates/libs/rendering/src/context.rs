use bevy_ecs::prelude::Resource;
use avalanche_hlvk::{CommandBuffer, CommandPool, Context};

#[derive(Resource)]
pub struct RenderingContext {
    pub context: Context,
    pub command_pool: CommandPool,
    pub swapchain_command_buffer: Vec<CommandBuffer>,
}
