use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use bevy_app::{App, Plugin};
use bevy_ecs::prelude::{Entity, IntoSystemConfigs, Query, ResMut};
use bevy_ecs::system::Resource;
use bevy_log::info;
use bevy_utils::EntityHashMap;
use avalanche_hlvk::{Surface, Swapchain};
use avalanche_window::{HandleWrapper, WindowComponent};
use crate::{ExtractSchedule, Render, RenderApp, RenderSet};
use crate::prelude::Extract;

pub struct WindowRenderPlugin;

impl Plugin for WindowRenderPlugin {
    fn build(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
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
    windows: Extract<Query<(Entity, &WindowComponent)>>,
) {
    windows
        .iter()
        .for_each(|(entity, component)| {
        });
}

fn prepare_windows() {}
