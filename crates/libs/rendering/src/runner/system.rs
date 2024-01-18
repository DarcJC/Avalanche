use std::time::Duration;
use bevy_ecs::prelude::{Mut, World};
use bevy_log::error;
use bevy_utils::tracing::info_span;
use crate::extract::FrameContext;
use crate::prelude::RenderGraph;
use crate::prelude::window::ExtractedWindows;
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

    {
        let _span = info_span!("present_frames").entered();
        
        let windows = world.resource::<ExtractedWindows>();
        for window in windows.values() {
            if let Ok(image) = window.swapchain.acquire_next_image(Duration::from_secs_f32(0.033), None) {
                if !image.is_suboptimal {
                    let semaphore = frame_context.frame_finish_semaphore();
                    let queue = frame_context.render_context().present_queue.clone();
                    let _ = window.swapchain.queue_present(image.index, &[semaphore.as_ref()], &queue);
                }
            }
        }
    }
}
