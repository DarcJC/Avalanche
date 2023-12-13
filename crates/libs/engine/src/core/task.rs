mod main;

pub use main::*;
use bevy_app::{PluginGroup, PluginGroupBuilder};
use bevy_ecs::schedule::ScheduleLabel;

#[derive(ScheduleLabel, Hash, PartialEq, Eq, Debug, Clone)]
struct AvalancheDefaultSchedule;

pub struct MinimalPlugins;

impl PluginGroup for MinimalPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(bevy_app::MainSchedulePlugin)
            .add(bevy_core::TaskPoolPlugin::default())
            .add(bevy_core::TypeRegistrationPlugin)
            .add(bevy_core::FrameCountPlugin)
            .add(bevy_time::TimePlugin)
            .add(bevy_app::ScheduleRunnerPlugin::default())
    }
}
