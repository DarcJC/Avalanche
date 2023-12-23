use std::io::Write;
use std::ops::Deref;
use std::sync::RwLock;
use bevy_app::{App, Plugin, PluginGroup, PluginGroupBuilder, PostStartup, PreStartup};
use bevy_ecs::prelude::{Resource, World};
use chrono::Local;
use ash::vk;
use env_logger::Env;
use avalanche_hlvk::{CommandBuffer, CommandPool, Context, ContextBuilder, DeviceFeatures, Swapchain};
use avalanche_window::{new_window_component, WindowManager, WindowSystemPlugin};

pub struct EngineContextSetupPlugin;

#[derive(Resource)]
pub struct RenderingContext {
    pub context: Context,
    pub command_pool: CommandPool,
    pub swapchain_command_buffer: Vec<CommandBuffer>,
}

/// Exclusive system to force schedule in main thread
fn start_rendering_system_with_window(world: &mut World) {
    let window_manager = world.get_non_send_resource::<WindowManager>().unwrap();
    let mut first_window_component = new_window_component(window_manager.event_loop.borrow_mut().deref()).unwrap();
    let window_ref = first_window_component.window.write().unwrap();

    let vulkan_context = ContextBuilder::new(window_ref.deref(), window_ref.deref())
        .required_device_features(DeviceFeatures::full())
        .with_raytracing_context(false)
        .app_name("Avalanche Engine")
        .required_device_extensions(vec!["VK_KHR_swapchain"].deref())
        .vulkan_version(avalanche_utils::VERSION_1_3)
        .build().unwrap();

    let command_pool = vulkan_context.create_command_pool(
        vulkan_context.graphics_queue_family,
        Some(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
    ).unwrap();

    let swapchain = Swapchain::new(
        &vulkan_context,
        window_ref.inner_size().width,
        window_ref.inner_size().height,
    ).unwrap();

    // TODO raytracing

    drop(window_ref);

    let command_buffers = command_pool.allocate_command_buffers(vk::CommandBufferLevel::PRIMARY, swapchain.images.len() as _).unwrap();

    first_window_component.render_device = Some(vulkan_context.device.clone());
    first_window_component.surface = Some(vulkan_context.surface.clone());
    first_window_component.swapchain = Some(RwLock::new(swapchain));

    let context = RenderingContext {
        context: vulkan_context,
        command_pool,
        swapchain_command_buffer: command_buffers,
    };

    world.insert_resource(context);
    world.spawn(first_window_component);
}

impl Plugin for EngineContextSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, start_rendering_system_with_window);
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
            .add(WindowSystemPlugin)
            .add(EngineContextSetupPlugin)
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
