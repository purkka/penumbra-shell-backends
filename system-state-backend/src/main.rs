mod state;
mod sysfs;
mod watch;

use futures::StreamExt;
use log::info;

use crate::state::{SystemState, SystemStatePart};
use crate::sysfs::Brightness;
use crate::watch::watch_file;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let (brightness_path, initial_brightness, max_brightness) =
        Brightness::initialize().expect("Failed to detect and initialize backlight brightness");

    let mut system_state = SystemState::initialize(initial_brightness, max_brightness);

    info!("initial state: {system_state:?}");

    let mut stream = watch_file(brightness_path.as_ref()).await?;

    while let Some(event) = stream.next().await {
        system_state.apply(event);
        info!("new state: {system_state:?}");
    }

    Ok(())
}
