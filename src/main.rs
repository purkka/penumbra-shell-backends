mod niri_ipc;

use crate::niri_ipc::NiriIPCClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut _client = NiriIPCClient::connect().await?;

    println!("Client connected");

    Ok(())
}
