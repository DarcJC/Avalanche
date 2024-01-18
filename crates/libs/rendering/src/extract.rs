mod frame;
pub use frame::*;

use bevy_ecs::prelude::{World};
use crate::MainWorld;
use crate::prelude::RenderingContext;

pub(crate) fn extract_rendering_context(render_world: &mut World) {
    let main_world = render_world.resource::<MainWorld>();
    let rendering_context = main_world.get_resource::<RenderingContext>().unwrap();
    let rendering_context = rendering_context.clone();
    // SAFETY: running in exclusive system
    unsafe {
        render_world.insert_resource(FrameContext::new(rendering_context));
    }
}

pub(crate) fn _extract_scene() {}

pub(crate) fn release_referenced_rendering_context(world: &mut World) {
    let context = world.remove_resource::<FrameContext>().unwrap();
    let _ = context.sync_fence_ref().wait(None);
    //context.render_context.device_wait_idle().unwrap();
}
