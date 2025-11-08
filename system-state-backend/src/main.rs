mod manager;
mod state;
mod sysfs;
mod watch;

use crate::manager::SystemStateManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let manager = SystemStateManager::new();
    manager.listen_to_streams().await?;

    Ok(())
}
