use common::PrintStateInfo;
use serde::Serialize;

use crate::state::SystemState;

#[derive(Serialize)]
struct SystemStateInfo {
    brightness: u8, // percent
}

impl PrintStateInfo for SystemState {
    fn print_state_info(&self) -> anyhow::Result<()> {
        let brightness = ((self.brightness.brightness as u32 * 100)
            / self.brightness.max_brightness as u32) as u8;

        let state_info = SystemStateInfo { brightness };

        let state_info_json = serde_json::to_string(&state_info)?;
        println!("{state_info_json}");

        Ok(())
    }
}
