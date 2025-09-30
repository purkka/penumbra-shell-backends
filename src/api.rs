use std::collections::{BTreeMap, HashMap};

use itertools::Itertools;
use niri_ipc::{state::EventStreamState, Window, Workspace};
use serde::Serialize;

#[derive(Serialize)]
struct WorkspaceInfo {
    active_workspace: u8, // idx
    workspaces: Vec<u8>,
}

type WorkspacesInfo = HashMap<String, WorkspaceInfo>; // map from output to info

#[derive(Serialize)]
struct WorkspaceWindowsInfo {
    focused_window_id: Option<u64>,
    window_ids: Vec<u64>,
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

                let nof_workspaces: u8 = workspaces.len().try_into().unwrap();
                let workspaces: Vec<u8> = (1..(nof_workspaces + 1)).collect();

                (
                    output.clone(),
                    WorkspaceInfo {
                        active_workspace,
                        workspaces,
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
                let scrolling_layout_window_map: BTreeMap<usize, Vec<&Window>> =
                    all_windows_grouped
                        .get(workspace_id)
                        .map(|windows| {
                            // create btree with x-position (column) as the key
                            let mut map: BTreeMap<usize, Vec<&Window>> = windows
                                .iter()
                                .filter_map(|wd| {
                                    wd.layout.pos_in_scrolling_layout.map(|(x, _)| (x, wd))
                                })
                                .fold(
                                    BTreeMap::<usize, Vec<&Window>>::new(),
                                    |mut acc, (x, wd)| {
                                        acc.entry(x).or_default().push(wd);
                                        acc
                                    },
                                );

                            // sort each column according to their y-positions as well
                            map.values_mut().for_each(|column| {
                                column
                                    .sort_by_key(|wd| wd.layout.pos_in_scrolling_layout.unwrap().1)
                            });

                            map
                        })
                        .unwrap_or_default();

                let scrolling_layout_window_ids: Vec<u64> = scrolling_layout_window_map
                    .values()
                    .filter_map(|column| column.first().map(|wd| wd.id))
                    .collect();

                let scrolling_layout_focused_window: Option<&Window> = scrolling_layout_window_map
                    .values()
                    .flat_map(|column| column.iter().copied())
                    .find(|wd| wd.is_focused);

                match scrolling_layout_focused_window {
                    Some(focused_window) => {
                        let focused_column_topmost_id: Option<u64> =
                            scrolling_layout_window_map.iter().find_map(|(_, col)| {
                                if col.iter().any(|w| w.id == focused_window.id) {
                                    col.first().map(|w| w.id)
                                } else {
                                    None
                                }
                            });

                        (
                            output.clone(),
                            WorkspaceWindowsInfo {
                                focused_window_id: focused_column_topmost_id,
                                window_ids: scrolling_layout_window_ids,
                            },
                        )
                    }
                    None => (
                        output.clone(),
                        WorkspaceWindowsInfo {
                            focused_window_id: None,
                            window_ids: scrolling_layout_window_ids,
                        },
                    ),
                }
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
