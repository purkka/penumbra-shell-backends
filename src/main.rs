use niri_workspaces_rs::client::ClientManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manager = ClientManager::new().await;
    manager.listen_to_event_stream().await?;

    Ok(())
}
