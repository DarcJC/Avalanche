use std::io::Write;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use bevy_app::{App, Plugin, PluginGroup, PluginGroupBuilder, Update};
use bevy_ecs::prelude::{EventReader, IntoSystemConfigs, IntoSystemSetConfigs, Query, Res, World};
use chrono::Local;
use ash::vk;
use bevy_ecs::event::EventWriter;
use env_logger::Env;
use log::warn;
use avalanche_hlvk::{ContextBuilder, DeviceFeatures, Swapchain};
use avalanche_rendering::prelude::RenderingContext;
use avalanche_rendering::{INIT_COMMAND_POOL_NUM, RenderingPipelinePlugin, RenderSet};
use avalanche_window::{new_window_component, WindowComponent, WindowManager, WindowSystemPlugin, WindowSystemSet};
use avalanche_window::event::{WindowEventLoopClearedEvent, WindowResizedEvent};
use crate::core::event::BeginRenderWindowViewEvent;

pub struct EngineContextSetupPlugin;

/// Exclusive system to force schedule in main thread
fn start_rendering_system_with_window(world: &mut World) {
    let window_manager = world.get_non_send_resource::<WindowManager>().unwrap();
    let mut first_window_component = new_window_component(window_manager.event_loop.borrow_mut().deref()).unwrap();
    let window_ref = &first_window_component.window;

    let vulkan_context = ContextBuilder::new(window_ref, window_ref)
        .required_device_features(DeviceFeatures::full())
        .with_raytracing_context(false)
        .app_name("Avalanche Engine")
        .required_device_extensions(vec!["VK_KHR_swapchain"].deref())
        .vulkan_version(avalanche_utils::VERSION_1_3)
        .build().unwrap();

    let command_pools = (0..INIT_COMMAND_POOL_NUM)
        .map(|_| Arc::new(vulkan_context.create_command_pool(
            vulkan_context.graphics_queue_family,
            Some(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        ).unwrap()))
        .collect::<Vec<_>>();

    let swapchain = Swapchain::new(
        &vulkan_context,
        window_ref.inner_size().width,
        window_ref.inner_size().height,
    ).unwrap();

    // TODO raytracing

    first_window_component.render_device = Some(vulkan_context.device.clone());
    first_window_component.surface = Some(vulkan_context.surface.clone());
    first_window_component.swapchain = Some(Arc::new(RwLock::new(swapchain)));

    let context = RenderingContext {
        context: Arc::new(vulkan_context),
        command_pools: Arc::new(command_pools),
    };

    world.insert_resource(context);
    world.spawn(first_window_component);
}

fn window_resize_handler(mut event_reader: EventReader<WindowResizedEvent>, windows: Query<&WindowComponent>, rendering_context: Res<RenderingContext>) {
    #[cfg(feature = "trace")]
    let _span = bevy_utils::tracing::info_span!("window swapchain recreated").entered();

    event_reader.read().for_each(|evt|  {
        if let Some(window) = windows
            .iter()
            .find(|i| i.window.id() == evt.window_id)  {
            let res = window.swapchain
                .as_ref()
                .unwrap()
                .write()
                .unwrap()
                .resize(&rendering_context.context, evt.new_size.0, evt.new_size.1);
            if let Err(err) = res {
                warn!("[Window] Failed to recreate swapchain for window: {err}");
            }
        }
    })
}

fn window_event_loop_cleared(mut event_reader: EventReader<WindowEventLoopClearedEvent>, _event_sender: EventWriter<BeginRenderWindowViewEvent>, _windows: Query<&WindowComponent>, _rendering_context: Res<RenderingContext>) {
    #[cfg(feature = "trace")]
    let _span = bevy_utils::tracing::info_span!("window present queued").entered();

    // TODO: must be sure to run once pre frame
    if event_reader.read().is_empty() {
        return;
    }
}

impl Plugin for EngineContextSetupPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, (
                WindowSystemSet::EventLoop,
                WindowSystemSet::Update,
                RenderSet::Notify,
            ).chain());
        // app.add_systems(PostStartup, start_rendering_system_with_window);
        app.add_systems(Update, (
            window_resize_handler
                .after(WindowSystemSet::Update)
                .before(RenderSet::Notify),
            window_event_loop_cleared
                .after(window_resize_handler)
                .after(WindowSystemSet::Update)
                .in_set(RenderSet::Notify),
        ));
        app.add_event::<BeginRenderWindowViewEvent>();
        start_rendering_system_with_window(&mut app.world);
    }
}

pub struct LogSystemPlugin;

fn _init_env_logger() {
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
        use bevy_log::LogPlugin;
        app.add_plugins(LogPlugin::default());
    }
}

pub struct MainTaskPluginGroup;

impl PluginGroup for MainTaskPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        let mut builder = PluginGroupBuilder::start::<Self>()
            .add(LogSystemPlugin)
            .add(WindowSystemPlugin)
            .add(EngineContextSetupPlugin)
            .add(RenderingPipelinePlugin);

        #[cfg(feature = "renderdoc")]
        {
            builder = builder.add(avalanche_rendering::prelude::renderdoc::RenderDocPlugin);
        }

        builder
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
