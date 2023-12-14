mod main;

pub use main::*;
use bevy_app::{PluginGroup, PluginGroupBuilder};
use bevy_ecs::schedule::ScheduleLabel;

#[derive(ScheduleLabel, Hash, PartialEq, Eq, Debug, Clone)]
struct AvalancheDefaultSchedule;
