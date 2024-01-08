use std::time::Duration;
use ash::vk;
use bevy_ecs::prelude::Res;
use avalanche_hlvk::SemaphoreSubmitInfo;
use crate::preclude::RenderingContext;
use crate::present::FramesInFlight;

/// Using to verify does raster pipeline working
pub(crate) fn clear_screen_color(context: Res<RenderingContext>, frames_in_flight: Res<FramesInFlight>) {
    let frame = frames_in_flight.0.first().unwrap();
    let extent = frame.swapchain_image.extent;

    let command_buffer = context.swapchain_command_buffer.first().unwrap();

    let view = frame.swapchain_image.create_image_view().unwrap();
    let clear_color: [f32; 4] = [0.33, 0.33, 0.33, 1.0];
    command_buffer.begin(None).unwrap();
    command_buffer.begin_rendering(&view, vk::Extent2D{ width: extent.width, height: extent.height }, vk::AttachmentLoadOp::CLEAR, Some(clear_color));

    command_buffer.end_rendering();
    command_buffer.end().unwrap();

    context.context.graphics_queue.submit(command_buffer, None, Some(SemaphoreSubmitInfo {
        semaphore: frame.frame_finish_semaphore.as_ref(),
        stage_mask: vk::PipelineStageFlags2::ALL_GRAPHICS,
    }), frame.working_fence.as_ref()).unwrap();

    command_buffer.reset().unwrap();
    frame.working_fence.wait(Some(Duration::from_secs_f32(0.33))).unwrap();
    frame.working_fence.reset().unwrap();
}
