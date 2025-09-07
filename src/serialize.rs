use std::collections::HashMap;

use niri_ipc::{
    state::{
        ConfigState, EventStreamState, KeyboardLayoutsState, OverviewState, WindowsState,
        WorkspacesState,
    },
    KeyboardLayouts, Window, Workspace,
};
use serde::Serialize;

#[derive(Serialize)]
#[serde(remote = "EventStreamState")]
struct EventStreamStateDef {
    #[serde(with = "WorkspacesStateDef")]
    workspaces: WorkspacesState,
    #[serde(with = "WindowsStateDef")]
    windows: WindowsState,
    #[serde(with = "KeyboardLayoutsStateDef")]
    keyboard_layouts: KeyboardLayoutsState,
    #[serde(with = "OverviewStateDef")]
    overview: OverviewState,
    #[serde(with = "ConfigStateDef")]
    config: ConfigState,
}

#[derive(Serialize)]
#[serde(remote = "WorkspacesState")]
pub struct WorkspacesStateDef {
    pub workspaces: HashMap<u64, Workspace>,
}

#[derive(Serialize)]
#[serde(remote = "WindowsState")]
pub struct WindowsStateDef {
    pub windows: HashMap<u64, Window>,
}

#[derive(Serialize)]
#[serde(remote = "KeyboardLayoutsState")]
pub struct KeyboardLayoutsStateDef {
    pub keyboard_layouts: Option<KeyboardLayouts>,
}

#[derive(Serialize)]
#[serde(remote = "OverviewState")]
pub struct OverviewStateDef {
    pub is_open: bool,
}

#[derive(Serialize)]
#[serde(remote = "ConfigState")]
pub struct ConfigStateDef {
    pub failed: bool,
}
