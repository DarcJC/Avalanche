use std::io::Write;
use bevy_app::{App, Plugin, PluginGroup, PluginGroupBuilder, PostStartup, PreStartup, Update};
use bevy_ecs::prelude::World;
use chrono::Local;
use env_logger::Env;
use log::info;
use avalanche_window::get_window_manager_mut_sync;

pub struct WindowSystemTaskPlugin;

/// Exclusive system to force schedule in main thread
fn start_window_system(_world: &mut World) {
    let _ = get_window_manager_mut_sync().create_main_window();
    info!("Main window created.");
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
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {} - ({}:{})",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args(),
                record.module_path().unwrap_or("unknown"),
                record.line().map_or(-1, |value| value as i32)
            )
        })
        .init();
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

pub struct SchedulerMinimalPlugins;

impl PluginGroup for SchedulerMinimalPlugins {
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
