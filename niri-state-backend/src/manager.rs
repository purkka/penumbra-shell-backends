use std::pin::Pin;

use common::{PrintStateInfo, StateManager};
use futures::StreamExt;
use log::{debug, info};

use crate::{client::NiriIPCClient, state::NiriState};

pub struct NiriStateManager {
    client: NiriIPCClient,
    state: NiriState,
}

impl NiriStateManager {
    pub async fn new() -> Self {
        Self {
            client: NiriIPCClient::connect()
                .await
                .expect("Failed to connect to niri IPC"),
            state: NiriState::default(),
        }
    }
}

impl StateManager for NiriStateManager {
    fn listen_to_stream(
        mut self: Box<Self>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>> {
        Box::pin(async move {
            let mut events = Box::pin(self.client.request_and_read_event_stream().await);

            while let Some(Ok(event)) = events.next().await {
                info!("Received event: {event:?}");

                self.state.apply(event);
                debug!("New state: {0:?}", self.state);

                self.state.print_state_info()?;
            }

            Ok(())
        })
    }
}
