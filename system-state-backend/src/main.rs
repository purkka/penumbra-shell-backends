mod state;
mod watch;

use std::fs;
use std::path::{Path, PathBuf};

use futures::StreamExt;
use log::info;

use crate::state::{SystemState, SystemStatePart};
use crate::watch::watch_file;

const BACKLIGHT: &str = "/sys/class/backlight";

fn detect_backlight_device() -> Option<PathBuf> {
    let base = Path::new(BACKLIGHT);
    if !base.exists() {
        return None;
    }

    if let Ok(entries) = fs::read_dir(base) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.join("brightness").exists() && path.join("max_brightness").exists() {
                return Some(path);
            }
        }
    }
    None
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let backlight_device = detect_backlight_device()
        .unwrap_or_else(|| panic!("No valid backlight found in {BACKLIGHT}"));

    let brightness_path = backlight_device.join("brightness");
    let max_brightness_path = backlight_device.join("max_brightness");

    let max_brightness: u16 = fs::read_to_string(&max_brightness_path)?
        .trim()
        .parse()
        .unwrap();
    let current_brightness: u16 = fs::read_to_string(&brightness_path)?
        .trim()
        .parse()
        .unwrap();

    let mut system_state = SystemState::initialize(current_brightness, max_brightness);

    info!("initial state: {system_state:?}");

    let mut stream = watch_file(brightness_path.as_ref()).await?;

    while let Some(event) = stream.next().await {
        system_state.apply(event);
        info!("new state: {system_state:?}");
    }

    Ok(())
}
