use std::env;

use anyhow::anyhow;
use futures::{stream, Stream, StreamExt};
use log::{debug, info};
use niri_ipc::{
    state::{EventStreamState, EventStreamStatePart},
    Event, Reply, Request, Response,
};
use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{unix, UnixStream},
};

use crate::eww;

pub struct NiriIPCClient {
    reader: BufReader<unix::OwnedReadHalf>,
    writer: BufWriter<unix::OwnedWriteHalf>,
}

impl NiriIPCClient {
    pub async fn connect() -> anyhow::Result<Self> {
        let socket_path =
            env::var("NIRI_SOCKET").map_err(|e| anyhow!(e).context("NIRI_SOCKET unset"))?;
        let unixstream = UnixStream::connect(socket_path).await?;
        let (read_half, write_half) = unixstream.into_split();

        Ok(Self {
            reader: BufReader::new(read_half),
            writer: BufWriter::new(write_half),
        })
    }

    pub async fn send(&mut self, request: Request) -> io::Result<Reply> {
        let mut buf = serde_json::to_string(&request).unwrap();
        buf.push('\n');

        self.writer.write_all(buf.as_bytes()).await?;
        self.writer.flush().await?;

        buf.clear();
        self.reader.read_line(&mut buf).await?;

        let reply = serde_json::from_str(&buf)?;

        Ok(reply)
    }

    pub async fn read_into_event_stream(self) -> impl Stream<Item = io::Result<Event>> {
        // shutdown writer
        if let Err(e) = self.writer.into_inner().shutdown().await {
            eprintln!("Shutting down writer failed: {e:?}")
        }

        stream::unfold(self.reader, |mut reader| async move {
            let mut buf = String::new();
            match reader.read_line(&mut buf).await {
                Ok(0) => None,
                Ok(_) => {
                    let event = serde_json::from_str(&buf)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e));
                    buf.clear();
                    Some((event, reader))
                }
                Err(e) => Some((Err(e), reader)),
            }
        })
    }
}

pub struct ClientManager {
    client: NiriIPCClient,
    state: EventStreamState,
}

impl ClientManager {
    pub async fn new() -> Self {
        Self {
            client: NiriIPCClient::connect()
                .await
                .expect("Failed to connect to niri IPC"),
            state: EventStreamState::default(),
        }
    }

    pub async fn listen_to_event_stream(mut self) -> anyhow::Result<()> {
        match self.client.send(Request::EventStream).await? {
            Ok(Response::Handled) => info!("Requested event stream succesfully"),
            Ok(other) => panic!("Unexpected response from niri: {other:?}"),
            Err(msg) => panic!("Niri returned error: {msg}"),
        }

        let events = self.client.read_into_event_stream().await;
        let mut events = Box::pin(events);

        while let Some(Ok(event)) = events.next().await {
            info!("Received event: {event:?}");

            self.state.apply(event);
            debug!("New state: {0:?}", self.state);

            eww::yuckify(&self.state);
        }

        Ok(())
    }
}
