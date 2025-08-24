use niri_workspaces_rs::niri_ipc::NiriIPCClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut _client = NiriIPCClient::connect().await?;

    println!("Client connected");

    Ok(())
}
