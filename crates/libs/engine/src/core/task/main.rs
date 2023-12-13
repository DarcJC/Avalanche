use bevy_app::{App, Plugin, PluginGroup, PluginGroupBuilder, PostStartup, PreStartup, Update};
use bevy_ecs::prelude::World;
use log::{error, info};
use avalanche_window::get_window_manager_mut_sync;

pub struct WindowSystemTaskPlugin;

/// Exclusive system to force schedule in main thread
fn start_window_system(_world: &mut World) {
    let _ = get_window_manager_mut_sync().create_main_window();
}

fn poll_window_events(_world: &mut World) {
    get_window_manager_mut_sync().handle_events();
}

impl Plugin for WindowSystemTaskPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, start_window_system);
        app.add_systems(Update, poll_window_events);
    }
}

pub struct LogSystemPlugin;

fn init_env_logger() {
    env_logger::init();
}

impl Plugin for LogSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, init_env_logger);
    }
}

pub struct MainTaskPluginGroup;

impl PluginGroup for MainTaskPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(LogSystemPlugin)
            .add(WindowSystemTaskPlugin)
    }
}
