use bevy_ecs::prelude::World;
use crate::MainWorld;
use crate::preclude::RenderingContext;

pub(crate) fn extract_rendering_context(render_world: &mut World) {
    let main_world = render_world.resource::<MainWorld>();
    let rendering_context = main_world.get_resource::<RenderingContext>().unwrap();
    let rendering_context = rendering_context.clone();
    render_world.insert_resource(rendering_context);
}

pub(crate) fn _extract_scene() {}

pub(crate) fn release_referenced_rendering_context(world: &mut World) {
    world.remove_resource::<RenderingContext>();
}
