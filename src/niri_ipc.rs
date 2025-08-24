use std::{
    collections::HashMap,
    env,
    sync::{Arc, RwLock},
};

use niri_ipc::Event;
use tokio::{
    io::{BufReader, BufWriter},
    net::{unix, UnixStream},
};

use crate::handlers::EventHandler;

type HandlerMap = Arc<RwLock<HashMap<Event, Vec<Arc<dyn EventHandler>>>>>;

pub struct NiriIPCClient {
    reader: BufReader<unix::OwnedReadHalf>,
    writer: BufWriter<unix::OwnedWriteHalf>,
    handler_map: HandlerMap,
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
            handler_map: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}
