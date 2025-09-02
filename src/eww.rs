use niri_ipc::state::EventStreamState;

pub fn yuckify(state: &EventStreamState) {
    let active_workspace = state
        .workspaces
        .workspaces
        .iter()
        .filter(|(_id, workspace)| workspace.is_active)
        .take(1)
        .map(|(_id, workspace)| workspace.id)
        .collect::<Vec<u64>>();
    println!("(box (label :text \"{active_workspace:?}\"))");
}
