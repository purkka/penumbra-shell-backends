use std::fs;
use std::path::Path;

use log::error;
use notify::event::{AccessKind::Close, AccessMode::Write};
use notify::{Event, EventKind::Access, INotifyWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;

use crate::state::SystemEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemEventKind {
    Brightness,
}

fn inotify_watcher() -> notify::Result<(
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

pub async fn watch_file(
    path: &Path,
    system_event_kind: SystemEventKind,
) -> notify::Result<ReceiverStream<SystemEvent>> {
    let (mut watcher, rx) = inotify_watcher()?;
    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    let (app_tx, app_rx) = mpsc::channel::<SystemEvent>(10);

    let path = path.to_owned();

    tokio::spawn(async move {
        let _watcher = watcher; // keep watcher alive
        let mut rx = ReceiverStream::new(rx);

        while let Some(res) = rx.next().await {
            match res {
                Ok(Event { kind, .. }) => {
                    if matches!(kind, Access(Close(Write))) {
                        if let Ok(contents) = fs::read_to_string(&path) {
                            // TODO Handle other types of files as well
                            let system_event = match system_event_kind {
                                SystemEventKind::Brightness => SystemEvent::BrightnessChanged {
                                    new_brightness: contents.trim().parse().unwrap(),
                                },
                            };
                            let _ = app_tx.send(system_event).await;
                        }
                    }
                }
                Err(e) => error!("watch error: {e:?}"),
            }
        }
    });

    Ok(ReceiverStream::new(app_rx))
}
