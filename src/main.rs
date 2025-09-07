use niri_state_rs::client::ClientManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let manager = ClientManager::new().await;
    manager.listen_to_event_stream().await?;

    Ok(())
}
