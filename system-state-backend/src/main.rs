mod state;

use std::fs;
use std::path::{Path, PathBuf};

use futures::StreamExt;
use log::{error, info};
use notify::event::{AccessKind::Close, AccessMode::Write};
use notify::{Event, EventKind::Access, INotifyWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::state::{SystemEvent, SystemState, SystemStatePart};

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

fn async_watcher() -> notify::Result<(
    notify::INotifyWatcher,
    tokio::sync::mpsc::Receiver<notify::Result<notify::Event>>,
)> {
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>(1);
    let watcher = INotifyWatcher::new(
        move |res| {
            let _ = tx.blocking_send(res);
        },
        notify::Config::default(),
    )?;

    Ok((watcher, rx))
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

    let (mut watcher, rx) = async_watcher()?;

    watcher.watch(brightness_path.as_ref(), RecursiveMode::NonRecursive)?;

    let mut rx = ReceiverStream::new(rx);

    while let Some(res) = rx.next().await {
        match res {
            Ok(Event { kind, .. }) => {
                if matches!(kind, Access(Close(Write))) {
                    if let Ok(contents) = fs::read_to_string(&brightness_path) {
                        let system_event = SystemEvent::BrightnessChanged {
                            new_brightness: contents.trim().parse().unwrap(),
                        };

                        system_state.apply(system_event);

                        info!("new state: {system_state:?}");
                    }
                }
            }
            Err(e) => error!("watch error: {e:?}"),
        }
    }

    Ok(())
}
