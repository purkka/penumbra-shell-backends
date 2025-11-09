use std::collections::HashMap;

use niri_ipc::{
    state::{EventStreamState, EventStreamStatePart},
    Event, Window, Workspace,
};

/// A wrapper around `EventStreamState` to allow implementing `PrintStateInfo`
#[derive(Debug, Default)]
pub struct NiriState(pub EventStreamState);

impl NiriState {
    pub fn apply(&mut self, event: Event) -> Option<Event> {
        self.0.apply(event)
    }

    pub fn workspaces_state(&self) -> HashMap<u64, Workspace> {
        self.0.workspaces.workspaces.clone()
    }

    pub fn windows_state(&self) -> HashMap<u64, Window> {
        self.0.windows.windows.clone()
    }
}
