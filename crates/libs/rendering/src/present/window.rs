use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use ash::vk;
use bevy_app::{App, Plugin};
use bevy_ecs::change_detection::Res;
use bevy_ecs::prelude::{Entity, IntoSystemConfigs, Query, ResMut};
use bevy_ecs::system::Resource;
use bevy_utils::EntityHashMap;
use log::warn;
use winit::dpi::PhysicalSize;
use avalanche_hlvk::{Surface, Swapchain};
use avalanche_window::{HandleWrapper, PrimaryWindowComponent, WindowComponent};
use crate::{ExtractSchedule, Render, RenderApp, RenderSet};
use crate::extract::FrameContext;
use crate::prelude::Extract;

pub struct WindowRenderPlugin;

#[derive(Resource, Default)]
pub struct NonSendMark;

impl Plugin for WindowRenderPlugin {
    fn build(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_non_send_resource::<NonSendMark>()
                .init_resource::<ExtractedWindows>()
                .add_systems(ExtractSchedule, extract_windows)
                .add_systems(Render, prepare_windows.in_set(RenderSet::ManageViews));
        }
    }
}

pub struct ExtractedWindow {
    pub entity: Entity,
    pub handle: HandleWrapper,
    pub swapchain: Arc<Swapchain>,
    pub surface: Arc<Surface>,
    pub cached_physical_width: u32,
    pub cached_physical_height: u32,
    pub cached_present_mode: vk::PresentModeKHR,
    pub size_changed: bool,
    pub present_mode_changed: bool,
}

#[derive(Default, Resource)]
pub struct ExtractedWindows {
    pub primary: Option<Entity>,
    pub windows: EntityHashMap<Entity, ExtractedWindow>,
}

impl Deref for ExtractedWindows {
    type Target = EntityHashMap<Entity, ExtractedWindow>;

    fn deref(&self) -> &Self::Target {
        &self.windows
    }
}

impl DerefMut for ExtractedWindows {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.windows
    }
}

fn extract_windows(
    mut extracted_windows: ResMut<ExtractedWindows>,
    windows: Extract<Query<(Entity, &WindowComponent, Option<&PrimaryWindowComponent>)>>,
) {
    for (entity, window_component, is_primary_window) in windows.iter() {
        if window_component.swapchain.is_none() || window_component.surface.is_none() {
            // Window is not initialized yet
            continue;
        }

        if is_primary_window.is_some() {
            extracted_windows.primary = Some(entity);
        }

        let handle = window_component.window.as_ref().into();
        let swapchain = window_component.swapchain.as_ref().unwrap().clone();
        let surface = window_component.surface.as_ref().unwrap().clone();

        let PhysicalSize {
            height: new_height,
            width: new_width,
        } = window_component.window.inner_size().clamp(PhysicalSize::new(1, 1), PhysicalSize::new(8192, 8192));
        let present_mode = window_component.swapchain.as_ref().unwrap().present_mode;

        let extracted_window = extracted_windows.entry(entity).or_insert(ExtractedWindow {
            entity,
            handle,
            swapchain,
            surface,
            cached_physical_width: new_width,
            cached_physical_height: new_height,
            cached_present_mode: present_mode,
            size_changed: false,
            present_mode_changed: false,
        });

        extracted_window.size_changed = new_width != extracted_window.cached_physical_width
            || new_height != extracted_window.cached_physical_height;
        extracted_window.present_mode_changed = extracted_window.cached_present_mode != present_mode;

        if extracted_window.size_changed {
            extracted_window.cached_physical_width = new_width;
            extracted_window.cached_physical_height = new_height;
        }

        if extracted_window.present_mode_changed {
            extracted_window.cached_present_mode = present_mode;
        }
    }
}

fn prepare_windows(extracted_windows: ResMut<ExtractedWindows>, frame_context: Res<FrameContext>) {
    for (_entity, window) in extracted_windows.windows.iter() {
        #[cfg(feature = "trace")]
        let _span = bevy_utils::tracing::info_span!("window swapchain recreated").entered();

        if window.size_changed {
            if let Err(err) = window.swapchain
                .as_ref()
                .resize(frame_context.render_context(), window.cached_physical_width, window.cached_physical_height) {
                warn!("[Window] Failed to recreate swapchain for window: {err}");
            }
        }
    }

}
