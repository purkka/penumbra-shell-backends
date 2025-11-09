use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::anyhow;

fn detect_sysfs_device(class_dir: &str, required_files: &[&str]) -> anyhow::Result<PathBuf> {
    let base = Path::new(class_dir);
    if !base.exists() {
        return Err(anyhow!("Sysfs class drectory ({class_dir}) doesn't exist"));
    }

    let entries = fs::read_dir(base).map_err(anyhow::Error::from)?;

    entries
        .flatten()
        .map(|entry| entry.path())
        .find(|path| required_files.iter().all(|file| path.join(file).exists()))
        .ok_or_else(|| anyhow!("Can't find matching device in {class_dir}"))
}

pub trait SysfsWatch<T> {
    fn initialize() -> anyhow::Result<(PathBuf, T)>;
}

pub struct Brightness;

impl SysfsWatch<(u16, u16)> for Brightness {
    fn initialize() -> anyhow::Result<(PathBuf, (u16, u16))> {
        const BACKLIGHT: &str = "/sys/class/backlight";
        const BRIGHTNESS: &str = "brightness";
        const MAX_BRIGHTNESS: &str = "max_brightness";

        let backlight_device = detect_sysfs_device(BACKLIGHT, &[BRIGHTNESS, MAX_BRIGHTNESS])?;

        let brightness_path = backlight_device.join(BRIGHTNESS);
        let max_brightness_path = backlight_device.join(MAX_BRIGHTNESS);

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
