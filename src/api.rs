use std::collections::HashMap;

use itertools::Itertools;
use niri_ipc::{state::EventStreamState, Window, Workspace};
use serde::Serialize;

#[derive(Serialize)]
struct WorkspaceInfo {
    active_workspace: u8, // idx
    nof_workspaces: u64,
}

type WorkspacesInfo = HashMap<String, WorkspaceInfo>; // map from output to info

#[derive(Serialize)]
struct WorkspaceWindowsInfo {
    focused_window: Option<usize>, // position from left
    focused_window_floating: Option<bool>,
    nof_windows: usize,
}

type WindowsInfo = HashMap<String, WorkspaceWindowsInfo>; // map from output to info

#[derive(Serialize)]
struct StateInfo {
    workspaces: WorkspacesInfo,
    windows: WindowsInfo,
}

pub trait PrintStateInfo {
    fn print_state_info(&self) -> Result<(), anyhow::Error>;
}

impl PrintStateInfo for EventStreamState {
    fn print_state_info(&self) -> Result<(), anyhow::Error> {
        let all_workspaces_grouped: HashMap<String, Vec<Workspace>> = self
            .workspaces
            .workspaces
            .clone()
            .into_values()
            .filter_map(|ws| ws.output.clone().map(|output| (output, ws)))
            .into_group_map();

        let mut active_workspaces: Vec<(String, u64)> = vec![];

        let workspaces: WorkspacesInfo = all_workspaces_grouped
            .iter()
            .map(|(output, workspaces)| {
                let active_workspace = match workspaces.iter().find(|ws| ws.is_active) {
                    Some(ws_active) => {
                        active_workspaces.push((
                            ws_active
                                .output
                                .clone()
                                .expect("Active workspace was not associated to any output"),
                            ws_active.id,
                        ));
                        ws_active.idx
                    }
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

        let all_windows_grouped: HashMap<u64, Vec<Window>> = self
            .windows
            .windows
            .clone()
            .into_values()
            .filter_map(|ws| ws.workspace_id.map(|workspace_id| (workspace_id, ws)))
            .into_group_map();

        let windows: WindowsInfo = active_workspaces
            .iter()
            .map(|(output, workspace_id)| {
                let workspace_windows = all_windows_grouped
                    .get(workspace_id)
                    .map(|v| v.as_slice())
                    .unwrap_or(&[]);

                let nof_windows = workspace_windows.len();

                let (focused_window, focused_window_floating) =
                    match workspace_windows.iter().find(|window| window.is_focused) {
                        Some(window) => {
                            let position = window
                                .layout
                                .pos_in_scrolling_layout
                                .map(|(column_index, _)| column_index);
                            let floating = Some(window.is_floating);
                            (position, floating)
                        }
                        None => (None, None),
                    };

                (
                    output.clone(),
                    WorkspaceWindowsInfo {
                        focused_window,
                        focused_window_floating,
                        nof_windows,
                    },
                )
            })
            .collect();

        let state_info = StateInfo {
            workspaces,
            windows,
        };

        let state_info_json = serde_json::to_string(&state_info)?;
        println!("{state_info_json}");

        Ok(())
    }
}
