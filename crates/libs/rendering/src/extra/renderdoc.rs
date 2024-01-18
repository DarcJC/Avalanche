use bevy_app::{App, Plugin};
use bevy_ecs::prelude::{Resource, World};

#[cfg(feature = "renderdoc")]
type RenderDocApiVersion = renderdoc::V141;
#[cfg(feature = "renderdoc")]
type RdV = renderdoc::RenderDoc<RenderDocApiVersion>;

/// A wrapper for [renderdoc::RenderDoc] instance.
///
/// ## SAFETY
/// Using [RwLock] to protect [renderdoc::RenderDoc] from being sharing in different threads.
///
/// Using [Arc] to protect [renderdoc::RenderDoc] from being transferring in different threads.
///
/// [Arc] will keeping [RenderDoc] valid until all references are released.
#[derive(Resource, Clone)]
pub struct RenderDoc {
    #[cfg(feature = "renderdoc")]
    pub inner: std::sync::Arc<std::sync::RwLock<RdV>>,
}

impl RenderDoc {
    #[cfg(feature = "renderdoc")]
    pub(crate) fn new(rd: RdV) -> Self {
        Self {
            inner: std::sync::Arc::new(std::sync::RwLock::new(rd)),
        }
    }
}

unsafe impl Sync for RenderDoc {}

unsafe impl Send for RenderDoc {}

pub struct RenderDocPlugin;

impl Plugin for RenderDocPlugin {
    #[allow(unused_variables)]
    fn build(&self, app: &mut App) {
        #[cfg(feature = "renderdoc")]
        {
            use renderdoc::Version;
            use renderdoc::CaptureOption::*;
            use bevy_log::{info, warn};

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

                app
                    .add_systems(crate::ExtractSchedule, (
                        extract_to_render_app,
                    ));
            } else {
                warn!("Failed to connect to RenderDoc api (supported version: {:?}).", RenderDocApiVersion::VERSION);
                warn!("Result: {}", result.unwrap_err());
            }
        }
    }
}

#[allow(dead_code)]
fn extract_to_render_app(render_world: &mut World) {
    let main_world = render_world.resource::<crate::MainWorld>();
    let renderdoc = main_world.get_resource::<RenderDoc>().unwrap();
    let renderdoc = renderdoc.clone();
    render_world.insert_resource(renderdoc);
}
