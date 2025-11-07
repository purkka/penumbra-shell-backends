use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct Brightness;

impl Brightness {
    const BACKLIGHT: &str = "/sys/class/backlight";
    const BRIGHTNESS: &str = "brightness";
    const MAX_BRIGHTNESS: &str = "max_brightness";

    fn detect_backlight_device() -> Option<PathBuf> {
        let base = Path::new(Self::BACKLIGHT);
        if !base.exists() {
            return None;
        }

        if let Ok(entries) = fs::read_dir(base) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.join(Self::BRIGHTNESS).exists() && path.join(Self::MAX_BRIGHTNESS).exists()
                {
                    return Some(path);
                }
            }
        }
        None
    }

    pub fn initialize() -> anyhow::Result<(PathBuf, u16, u16)> {
        let backlight_device = Self::detect_backlight_device()
            .unwrap_or_else(|| panic!("No valid backlight found in {}", Self::BACKLIGHT));

        let brightness_path = backlight_device.join(Self::BRIGHTNESS);
        let max_brightness_path = backlight_device.join(Self::MAX_BRIGHTNESS);

        let max_brightness: u16 = fs::read_to_string(&max_brightness_path)?
            .trim()
            .parse()
            .unwrap();
        let initial_brightness: u16 = fs::read_to_string(&brightness_path)?
            .trim()
            .parse()
            .unwrap();

        Ok((brightness_path, initial_brightness, max_brightness))
    }
}
