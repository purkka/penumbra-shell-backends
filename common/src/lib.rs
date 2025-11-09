pub trait PrintStateInfo {
    fn print_state_info(&self) -> anyhow::Result<()>;
}
