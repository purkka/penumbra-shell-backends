use std::{path::PathBuf, pin::Pin};

use common::StateManager;
use log::{debug, info};
use tokio_stream::{StreamExt, StreamMap};

use crate::{
    state::SystemState,
    sysfs::Brightness,
    watch::{SystemEventKind, watch_file},
};

pub struct SystemStateManager {
    state: SystemState,
    watch_paths: Vec<(PathBuf, SystemEventKind)>,
}

impl SystemStateManager {
    pub fn new() -> Self {
        let (brightness_path, initial_brightness, max_brightness) =
            Brightness::initialize().expect("Failed to detect and initialize backlight brightness");

        Self {
            state: SystemState::initialize(initial_brightness, max_brightness),
            watch_paths: vec![(brightness_path, SystemEventKind::Brightness)],
        }
    }
}

impl StateManager for SystemStateManager {
    fn listen_to_stream(
        mut self: Box<Self>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>> {
        Box::pin(async move {
            let mut stream_map = StreamMap::new();
            for (watch_file_path, system_event_kind) in self.watch_paths {
                let stream = watch_file(watch_file_path.as_ref(), system_event_kind).await?;
                stream_map.insert(system_event_kind, stream);
            }

            while let Some((event_kind, event)) = stream_map.next().await {
                info!("Received event (of type): {event:?} ({event_kind:?})");

                self.state.apply(event_kind, event);
                debug!("New state: {0:?}", self.state);
            }

            Ok(())
        })
    }
}
