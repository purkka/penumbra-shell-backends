use std::collections::HashMap;

use itertools::Itertools;
use niri_ipc::{state::WorkspacesState, Workspace};
use serde::Serialize;

#[derive(Serialize)]
struct WorkspaceInfo {
    active_workspace: u8,
    nof_workspaces: u64,
}

type Workspaces = HashMap<String, WorkspaceInfo>;

pub fn from_state(state: &WorkspacesState) -> Result<(), anyhow::Error> {
    let grouped: HashMap<String, Vec<Workspace>> = state
        .workspaces
        .clone()
        .into_values()
        .filter_map(|ws| ws.output.clone().map(|output| (output, ws)))
        .into_group_map();

    let workspaces: Workspaces = grouped
        .iter()
        .map(|(output, workspaces)| {
            let active_workspace = match workspaces.iter().find(|ws| ws.is_active) {
                Some(ws_active) => ws_active.idx,
                None => panic!("No active workspace found for output: {output}"),
            };
            let nof_workspaces = workspaces.len().try_into().unwrap();
            (
                output.clone(),
                WorkspaceInfo {
                    active_workspace,
                    nof_workspaces,
                },
            )
        })
        .collect();

    let workspaces_json = serde_json::to_string(&workspaces)?;
    println!("{workspaces_json}");

    Ok(())
}
