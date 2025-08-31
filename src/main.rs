use niri_ipc::{Request, Response};
use niri_workspaces_rs::niri_ipc::NiriIPCClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = NiriIPCClient::connect().await?;

    println!("Client connected");

    match client.send(Request::EventStream).await? {
        Ok(Response::Handled) => println!("Requested event stream succesfully"),
        Ok(other) => panic!("Unexpected response from niri: {other:?}"),
        Err(msg) => panic!("Niri returned error: {msg}"),
    }

    Ok(())
}
