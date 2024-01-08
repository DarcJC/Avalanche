use std::sync::Arc;
use std::time::Duration;
use bevy_ecs::prelude::World;
use bevy_ecs::system::Resource;
use avalanche_hlvk::{AcquiredImage, Fence, Semaphore};
use avalanche_window::{WindowComponent, WindowId};
use crate::MainWorld;
use crate::preclude::RenderingContext;

#[derive(Resource)]
pub struct FramesInFlight(Vec<FrameInFlight>);

pub struct FrameInFlight {
    window_id: WindowId,
    frame_finish_semaphore: Arc<Semaphore>,
    image_acquire_semaphore: Arc<Semaphore>,
    window_image: AcquiredImage,
    working_fence: Arc<Fence>,
}

pub(crate) fn create_frame_in_flight(render_world: &mut World) {
    let mut main_world = render_world.get_resource_mut::<MainWorld>().unwrap();

    let mut windows = main_world.query::<&WindowComponent>();
    let rendering_context = main_world.get_resource::<RenderingContext>().unwrap();

    let mut frames_in_flight = vec![];

    windows
        .iter(main_world.as_ref())
        .for_each(|window| {
            let mut swapchain = window.swapchain.as_ref().unwrap().write().unwrap();

            let window_id = window.id.clone();
            let frame_finish_semaphore = Arc::new(Semaphore::new(rendering_context.context.device.clone()).unwrap());
            let image_acquire_semaphore = swapchain.current_acquire_semaphore();
            let window_image = swapchain.acquire_next_image(Duration::from_secs_f64(0.33), None).unwrap();
            let working_fence = Arc::new(Fence::new(rendering_context.context.device.clone(), None).unwrap());

            let frame = FrameInFlight {
                window_id,
                frame_finish_semaphore: frame_finish_semaphore.clone(),
                image_acquire_semaphore: image_acquire_semaphore.clone(),
                window_image,
                working_fence,
            };

            frames_in_flight.push(frame);

            // Present frame after image is ready and frame rendered
            swapchain.queue_present(window_image.index, &[image_acquire_semaphore.as_ref(), frame_finish_semaphore.as_ref()], &rendering_context.context.present_queue).unwrap();
        });

    main_world.insert_resource(FramesInFlight(frames_in_flight));
}

pub(crate) fn cleanup_frames_in_flight(render_world: &mut World) {
    render_world.remove_resource::<FramesInFlight>();
}
