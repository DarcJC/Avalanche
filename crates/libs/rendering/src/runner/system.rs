use bevy_ecs::prelude::{Mut, World};
use bevy_log::error;
use crate::extract::FrameContext;
use crate::prelude::RenderGraph;
use crate::runner::RenderGraphRunner;

pub fn render_system(world: &mut World) {
    world.resource_scope(|world, mut graph: Mut<RenderGraph>| {
        graph.update(world);
    });

    let graph = world.resource::<RenderGraph>();
    let frame_context = world.resource::<FrameContext>();
    let render_device = frame_context.device();
    let render_queue = frame_context.graphics_queue();

    if let Err(err) = RenderGraphRunner::run(
        graph,
        render_device.clone(),
        &render_queue,
        world,
        |_context| {}
    ) {
        error!("Error running render graph:");
        {
            let mut src: &dyn std::error::Error = &err;
            loop {
                error!("> {}", src);
                match src.source() {
                    Some(s) => src = s,
                    None => break,
                }
            }
        }

        panic!("Error running render graph: {err}");
    }
}
