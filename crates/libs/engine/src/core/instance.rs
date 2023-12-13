use std::sync::Mutex;
use bevy_app::{App, AppExit};
use bevy_ecs::prelude::AppTypeRegistry;
use crate::core::task::{MainTaskPluginGroup, MinimalPlugins};

static INSTANCE_EXIT_FLAG: Mutex<bool> = Mutex::new(false);

pub struct EngineInstance {
    app: App,
}

impl Default for EngineInstance {
    fn default() -> Self {
        let mut app = App::empty();
        app.init_resource::<AppTypeRegistry>();
        app.add_plugins(MinimalPlugins);
        app.add_event::<AppExit>();
        app.add_plugins(MainTaskPluginGroup);
        Self {
            app,
        }
    }
}

impl EngineInstance {
    pub fn run(&mut self) -> EngineExitStatus {
        loop {
            if INSTANCE_EXIT_FLAG.lock().unwrap().clone() {
                break;
            }

            self.app.run();
        }

        EngineExitStatus::Normal
    }
}

pub enum EngineExitStatus {
    Normal,
}
