use std::ops::{Deref, DerefMut};
use bevy_app::{App, AppLabel, Plugin, SubApp};
use bevy_ecs::prelude::{IntoSystemConfigs, IntoSystemSetConfigs, Resource, Schedule, SystemSet};
use bevy_ecs::schedule::ScheduleLabel;
use bevy_ecs::world::World;
use crate::extract::{extract_rendering_context, release_referenced_rendering_context};
use crate::mock::clear_screen_color;
use crate::present::{cleanup_frames_in_flight, create_frame_in_flight};

mod extract;
pub mod context;
pub mod preclude;
mod present;
mod mock;


/// Schedule which extract data from the main world and inserts it into the render world.
///
/// This step should be kept as short as possible to increase the "pipelining potential" for
/// running the next frame while rendering the current frame.
///
/// This schedule is run on the main world, but its buffers are not applied
/// until it is returned to the render world.
#[derive(ScheduleLabel, PartialEq, Eq, Debug, Clone, Hash)]
pub struct ExtractSchedule;

/// The labels of the default App rendering sets.
///
/// that runs immediately after the matching system set.
/// These can be useful for ordering, but you almost never want to add your systems to these sets.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum RenderSet {
    /// Using to sort with window system
    /// as we want to perform rendering after game logics.
    Notify,
    /// This is used for applying the commands from the [`ExtractSchedule`]
    ExtractCommands,
    /// Prepare assets that have been created/modified/removed this frame.
    PrepareAssets,
    /// Create any additional views such as those used for shadow mapping.
    ManageViews,
    /// Queue drawable entities as phase items in [`RenderPhase`](crate::render_phase::RenderPhase)s
    /// ready for sorting
    Queue,
    /// A sub-set within [`Queue`](RenderSet::Queue) where mesh entity queue systems are executed. Ensures `prepare_assets::<Mesh>` is completed.
    QueueMeshes,
    // TODO: This could probably be moved in favor of a system ordering abstraction in `Render` or `Queue`
    /// Sort the [`RenderPhases`](render_phase::RenderPhase) here.
    PhaseSort,
    /// Prepare render resources from extracted data for the GPU based on their sorted order.
    /// Create [`BindGroups`](render_resource::BindGroup) that depend on those data.
    Prepare,
    /// A sub-set within [`Prepare`](RenderSet::Prepare) for initializing buffers, textures and uniforms for use in bind groups.
    PrepareResources,
    /// Flush buffers after [`PrepareResources`](RenderSet::PrepareResources), but before ['PrepareBindGroups'](RenderSet::PrepareBindGroups).
    PrepareResourcesFlush,
    /// A sub-set within [`Prepare`](RenderSet::Prepare) for constructing bind groups, or other data that relies on render resources prepared in [`PrepareResources`](RenderSet::PrepareResources).
    PrepareBindGroups,
    /// Actual rendering happens here.
    /// In most cases, only the render backend should insert resources here.
    Render,
    /// Cleanup render resources here.
    Cleanup,
}

/// The main render schedule.
#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub struct Render;

impl Render {
    /// Sets up the base structure of the rendering [`Schedule`].
    ///
    /// The sets defined in this enum are configured to run in order.
    pub fn base_schedule() -> Schedule {
        use RenderSet::*;

        let mut schedule = Schedule::new(Self);

        schedule.configure_sets(
            (
                ExtractCommands,
                ManageViews,
                Queue,
                PhaseSort,
                Prepare,
                Render,
                Cleanup,
            ).chain(),
        );

        schedule.configure_sets((ExtractCommands, PrepareAssets, Prepare).chain());
        schedule.configure_sets(QueueMeshes.in_set(Queue)); //.after(prepare_assets::<Mesh>));
        schedule.configure_sets(
            (PrepareResources, PrepareResourcesFlush, PrepareBindGroups)
                .chain()
                .in_set(Prepare),
        );

        schedule
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, AppLabel)]
pub struct RenderApp;

/// The simulation [`World`] of the application, stored as a resource.
/// This resource is only available during [`ExtractSchedule`] and not
/// during command application of that schedule.
/// See [`Extract`] for more details.
#[derive(Resource, Default)]
pub struct MainWorld(World);

impl Deref for MainWorld {
    type Target = World;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MainWorld {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A "scratch" world used to avoid allocating new worlds every frame when
/// swapping out the [`MainWorld`] for [`ExtractSchedule`].
#[derive(Resource, Default)]
struct ScratchMainWorld(World);

pub struct RenderingPipelinePlugin;

impl Plugin for RenderingPipelinePlugin {
    fn build(&self, app: &mut App) {
        // SAFETY: plugin is build on main thread
        unsafe { initialize_render_app(app); }
    }

    fn ready(&self, _app: &App) -> bool {
        true
    }
}

/// SAFETY: must be called in main thread
unsafe fn initialize_render_app(app: &mut App) {
    app.init_resource::<ScratchMainWorld>();

    let mut render_app = App::empty();
    render_app.main_schedule_label = Render.intern();

    let mut extract_schedule = Schedule::new(ExtractSchedule);
    extract_schedule.set_apply_final_deferred(false);

    render_app
        .add_schedule(extract_schedule)
        .add_schedule(Render::base_schedule())
        .add_systems(
            ExtractSchedule, (
                extract_rendering_context.before(create_frame_in_flight),
                create_frame_in_flight,
            ),
        )
        .add_systems(
            Render, (clear_screen_color.in_set(RenderSet::Render),)
        )
        .add_systems(
            Render, (
                World::clear_entities.in_set(RenderSet::Cleanup),
                cleanup_frames_in_flight.in_set(RenderSet::Cleanup),
                release_referenced_rendering_context.in_set(RenderSet::Cleanup).after(cleanup_frames_in_flight),
            ),
        );

    let (sender, receiver) = bevy_time::create_time_channels();
    app.insert_resource(receiver);
    render_app.insert_resource(sender);

    app.insert_sub_app(RenderApp, SubApp::new(render_app, move |main_world, render_app| {
        #[cfg(feature = "trace")]
        let _span = bevy_utils::tracing::info_span!("rendering extract ticked").entered();

        // reserve all existing main world entities for use in render_app
        // they can only be spawned using `get_or_spawn()`
        let total_count = main_world.entities().total_count();

        assert_eq!(
            render_app.world.entities().len(),
            0,
            "An entity was spawned after the entity list was cleared last frame and before the extract schedule began. This is not supported",
        );

        // SAFETY: This is safe given the clear_entities call in the past frame and the assert above
        unsafe {
            render_app
                .world
                .entities_mut()
                .flush_and_reserve_invalid_assuming_no_entities(total_count);
        }

        tick(main_world, render_app);
    }));
}

/// Executes the [`ExtractSchedule`] step of the renderer.
/// This updates the render world with the extracted ECS data of the current frame.
fn tick(main_world: &mut World, render_app: &mut App) {
    let scratch_world = main_world.remove_resource::<ScratchMainWorld>().unwrap();
    let inserted_world = std::mem::replace(main_world, scratch_world.0);
    render_app.world.insert_resource(MainWorld(inserted_world));

    render_app.world.run_schedule(ExtractSchedule);

    let inserted_world = render_app.world.remove_resource::<MainWorld>().unwrap();
    let scratch_world = std::mem::replace(main_world, inserted_world.0);
    main_world.insert_resource(ScratchMainWorld(scratch_world));
}
