use futures::StreamExt;
use niri_ipc::{Request, Response};
use niri_workspaces_rs::client::NiriIPCClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = NiriIPCClient::connect().await?;

    println!("Client connected");

    match client.send(Request::EventStream).await? {
        Ok(Response::Handled) => println!("Requested event stream succesfully"),
        Ok(other) => panic!("Unexpected response from niri: {other:?}"),
        Err(msg) => panic!("Niri returned error: {msg}"),
    }

    let events = client.read_into_event_stream().await;
    let mut events = Box::pin(events);

    while let Some(Ok(event)) = events.next().await {
        println!("Received event: {event:?}")
    }

    Ok(())
}
