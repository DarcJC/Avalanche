use std::sync::Arc;
use arc_swap::ArcSwap;
use bevy_app::{App, Plugin};
use bevy_ecs::prelude::Resource;
use bevy_log::{info, warn};
use renderdoc::CaptureOption::AllowFullscreen;

#[cfg(feature = "renderdoc")]
type RenderDocApiVersion = renderdoc::V141;
#[cfg(feature = "renderdoc")]
type RdV = renderdoc::RenderDoc<RenderDocApiVersion>;

/// A wrapper for [renderdoc::RenderDoc] instance.
///
/// ## SAFETY
/// Using [ArcSwap] to protect [renderdoc::RenderDoc] from being sharing in different threads.
///
/// Using [ArcSwap] to protect [renderdoc::RenderDoc] from being transferring in different threads.
///
/// [Arc] will keeping [RenderDoc] valid until all references are released.
#[derive(Resource)]
pub struct RenderDoc {
    #[cfg(feature = "renderdoc")]
    pub inner: ArcSwap<RdV>,
}

impl RenderDoc {
    #[cfg(feature = "renderdoc")]
    pub(crate) fn new(rd: RdV) -> Self {
        Self {
            inner: ArcSwap::new(Arc::new(rd)),
        }
    }
}

unsafe impl Sync for RenderDoc {}

unsafe impl Send for RenderDoc {}

pub struct RenderDocPlugin;

impl Plugin for RenderDocPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "renderdoc")]
        {
            use renderdoc::Version;
            use renderdoc::CaptureOption::*;

            let result = RdV::new();
            if result.is_ok() {
                let mut instance = result.unwrap();
                instance.set_capture_option_u32(AllowFullscreen, 1);
                instance.set_capture_option_u32(ApiValidation, 1);
                instance.set_capture_option_u32(CaptureCallstacks, 1);
                instance.set_capture_option_u32(CaptureCallstacksOnlyDraws, 1);
                instance.set_capture_option_u32(HookIntoChildren, 1);
                instance.set_capture_option_u32(CaptureAllCmdLists, 1);
                instance.set_capture_option_u32(DebugOutputMute, 1);
                instance.set_capture_option_u32(AllowUnsupportedVendorExtensions, 1);

                app.world.insert_resource(RenderDoc::new(instance));
                info!("Connected to RenderDoc api (version: {:?}).", RenderDocApiVersion::VERSION);
            } else {
                warn!("Failed to connect to RenderDoc api (supported version: {:?}).", RenderDocApiVersion::VERSION);
                warn!("Result: {}", result.unwrap_err());
            }
        }
    }
}
