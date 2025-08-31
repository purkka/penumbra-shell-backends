use std::env;

use niri_ipc::{Reply, Request};
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
            env::var("NIRI_SOCKET").expect("Niri is not running, niri IPC will not be available.");
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
        // TODO Read stream instead
        self.reader.read_line(&mut buf).await?;

        let reply = serde_json::from_str(&buf)?;

        Ok(reply)
    }
}
