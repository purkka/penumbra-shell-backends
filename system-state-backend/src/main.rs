mod api;
mod manager;
mod state;
mod sysfs;
mod watch;

use common::StateManager;

use crate::manager::SystemStateManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let manager = SystemStateManager::new();
    Box::new(manager).listen_to_stream().await?;

    Ok(())
}
