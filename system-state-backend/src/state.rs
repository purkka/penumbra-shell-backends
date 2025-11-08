// Design and implementation inspired by the event stream and state in
// `niri_ipc` from niri https://github.com/YaLTeR/niri.

use log::debug;

pub enum SystemEvent {
    BrightnessChanged { new_brightness: u16 },
}

pub trait SystemStatePart {
    /// Applies event to the state
    ///
    /// Returns `None` if event is applied. Returns `Some(SystemEvent)` if event is to be
    /// handled by another part of the state.
    fn apply(&mut self, event: SystemEvent) -> Option<SystemEvent>;
}

#[derive(Debug)]
struct BrightnessState {
    brightness: u16,
    max_brightness: u16,
}

impl SystemStatePart for BrightnessState {
    fn apply(&mut self, event: SystemEvent) -> Option<SystemEvent> {
        match event {
            SystemEvent::BrightnessChanged { new_brightness } => {
                self.brightness = new_brightness;
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct SystemState {
    brightness: BrightnessState,
}

impl SystemState {
    pub fn initialize(brightness: u16, max_brightness: u16) -> Self {
        let initial_state = Self {
            brightness: BrightnessState {
                brightness,
                max_brightness,
            },
        };

        debug!("initial state: {initial_state:?}");

        initial_state
    }
}

impl SystemStatePart for SystemState {
    fn apply(&mut self, event: SystemEvent) -> Option<SystemEvent> {
        let event = self.brightness.apply(event)?;
        Some(event)
    }
}
