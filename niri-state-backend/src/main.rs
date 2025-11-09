mod api;
mod client;
mod manager;
mod state;

use manager::NiriStateManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let manager = NiriStateManager::new().await;
    manager.listen_to_event_stream().await?;

    Ok(())
}
