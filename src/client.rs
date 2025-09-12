use std::env;

use anyhow::anyhow;
use futures::{
    stream::{self},
    Stream, StreamExt,
};
use log::{debug, info};
use niri_ipc::{
    state::{EventStreamStatePart, WorkspacesState},
    Event, Reply, Request, Response,
};
use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{unix, UnixStream},
};

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

    pub async fn request_and_read_event_stream(mut self) -> impl Stream<Item = io::Result<Event>> {
        match self.send(Request::EventStream).await {
            Ok(Ok(Response::Handled)) => info!("Requested event stream successfully"),
            Ok(Ok(other)) => panic!("Unexpected response from niri: {other:?}"),
            Ok(Err(msg)) => panic!("Niri returned error: {msg}"),
            Err(msg) => panic!("Transport error: {msg}"),
        }

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

pub struct ClientManager<T: EventStreamStatePart> {
    client: NiriIPCClient,
    state: T,
}

impl ClientManager<WorkspacesState> {
    pub async fn new() -> Self {
        Self {
            client: NiriIPCClient::connect()
                .await
                .expect("Failed to connect to niri IPC"),
            state: WorkspacesState::default(),
        }
    }

    pub async fn listen_to_event_stream(mut self) -> anyhow::Result<()> {
        let mut events = Box::pin(self.client.request_and_read_event_stream().await);

        while let Some(Ok(event)) = events.next().await {
            info!("Received event: {event:?}");

            match event {
                Event::WorkspacesChanged { .. }
                | Event::WorkspaceUrgencyChanged { .. }
                | Event::WorkspaceActivated { .. }
                | Event::WorkspaceActiveWindowChanged { .. } => {
                    self.state.apply(event);
                    debug!("New state: {0:?}", self.state);
                }
                _ => {}
            }
        }

        Ok(())
    }
}
