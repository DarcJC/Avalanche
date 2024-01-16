use std::ops::Deref;
use std::sync::Arc;
use bevy_ecs::prelude::Resource;
use avalanche_hlvk::{CommandPool, Context};

#[derive(Resource)]
pub struct RenderingContext {
    pub context: Arc<Context>,
    pub command_pools: Arc<Vec<Arc<CommandPool>>>,
}

impl Clone for RenderingContext {
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
            command_pools: self.command_pools.clone(),
        }
    }
}

impl Deref for RenderingContext {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        self.context.deref()
    }
}

impl RenderingContext {
}
