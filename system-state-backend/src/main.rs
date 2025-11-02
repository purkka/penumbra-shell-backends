use std::fs;
use std::path::{Path, PathBuf};

use futures::StreamExt;
use notify::{Event, INotifyWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

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
    let backlight_device = detect_backlight_device()
        .unwrap_or_else(|| panic!("No valid backlight found in {BACKLIGHT}"));

    let brightness_path = backlight_device.join("brightness");
    let max_brightness_path = backlight_device.join("max_brightness");

    let max_brightness: i32 = fs::read_to_string(&max_brightness_path)?
        .trim()
        .parse()
        .unwrap();
    let current: i32 = fs::read_to_string(&brightness_path)?
        .trim()
        .parse()
        .unwrap();

    println!("current brightness: {current} (max: {max_brightness})");

    let (mut watcher, rx) = async_watcher()?;

    watcher.watch(brightness_path.as_ref(), RecursiveMode::NonRecursive)?;

    let mut rx = ReceiverStream::new(rx);

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => println!("changed: {event:?}"),
            Err(e) => println!("watch error: {e:?}"),
        }
    }

    Ok(())
}
