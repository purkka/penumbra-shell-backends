use std::{
    fs,
    path::{Path, PathBuf},
};

fn detect_sysfs_device(class_dir: &str, required_files: &[&str]) -> Option<PathBuf> {
    let base = Path::new(class_dir);
    if !base.exists() {
        return None;
    }

    if let Ok(entries) = fs::read_dir(base) {
        return entries
            .flatten()
            .map(|entry| entry.path())
            .find(|path| required_files.iter().all(|file| path.join(file).exists()));
    }
    None
}

pub struct Brightness;

impl Brightness {
    const BACKLIGHT: &str = "/sys/class/backlight";
    const BRIGHTNESS: &str = "brightness";
    const MAX_BRIGHTNESS: &str = "max_brightness";

    pub fn initialize() -> anyhow::Result<(PathBuf, u16, u16)> {
        let backlight_device =
            detect_sysfs_device(Self::BACKLIGHT, &[Self::BRIGHTNESS, Self::MAX_BRIGHTNESS])
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
