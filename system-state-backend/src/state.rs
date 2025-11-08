use log::debug;

use crate::watch::SystemEventKind;

pub enum SystemEvent {
    BrightnessChanged { new_brightness: u16 },
}

#[derive(Debug)]
struct BrightnessState {
    brightness: u16,
    max_brightness: u16,
}

impl BrightnessState {
    fn update_brightness(&mut self, new_brightness: u16) {
        self.brightness = new_brightness;
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

    pub fn apply(&mut self, event_kind: SystemEventKind, event: SystemEvent) {
        match (event_kind, event) {
            (SystemEventKind::Brightness, SystemEvent::BrightnessChanged { new_brightness }) => {
                self.brightness.update_brightness(new_brightness)
            }
        }
    }
}
