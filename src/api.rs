use std::collections::HashMap;

use itertools::Itertools;
use niri_ipc::{state::EventStreamState, Workspace};
use serde::Serialize;

#[derive(Serialize)]
struct WorkspaceInfo {
    active_workspace: u8,
    nof_workspaces: u64,
}

type WorkspacesInfo = HashMap<String, WorkspaceInfo>; // map from output to info

#[derive(Serialize)]
struct StateInfo {
    workspaces: WorkspacesInfo,
}

pub trait PrintStateInfo {
    fn print_state_info(&self) -> Result<(), anyhow::Error>;
}

impl PrintStateInfo for EventStreamState {
    fn print_state_info(&self) -> Result<(), anyhow::Error> {
        let grouped: HashMap<String, Vec<Workspace>> = self
            .workspaces
            .workspaces
            .clone()
            .into_values()
            .filter_map(|ws| ws.output.clone().map(|output| (output, ws)))
            .into_group_map();

        let workspaces: WorkspacesInfo = grouped
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

        let state_info = StateInfo { workspaces };

        let state_info_json = serde_json::to_string(&state_info)?;
        println!("{state_info_json}");

        Ok(())
    }
}
