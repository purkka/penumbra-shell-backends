mod api;
mod client;
mod manager;
mod state;

use common::StateManager;
use manager::NiriStateManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let manager = NiriStateManager::new().await;
    Box::new(manager).listen_to_stream().await?;

    Ok(())
}
