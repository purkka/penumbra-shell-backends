use std::pin::Pin;

pub trait PrintStateInfo {
    fn print_state_info(&self) -> anyhow::Result<()>;
}

pub trait StateManager {
    fn listen_to_stream(
        self: Box<Self>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;
}
