// SPDX-License-Identifier: EUPL-1.2

use crate::libs::errors::T4lError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CustomPreset {
    pub name: String,
    pub pan: i32,
    pub tilt: i32,
    pub zoom: i32,
}

pub struct CustomPresetStore {
    path: PathBuf,
}

impl CustomPresetStore {
    pub fn new() -> Self {
        Self::with_path(presets_file_path())
    }

    fn with_path(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Vec<CustomPreset> {
        self.try_load().unwrap_or_default()
    }

    fn try_load(&self) -> Result<Vec<CustomPreset>, T4lError> {
        match fs::read(&self.path) {
            Ok(data) => serde_json::from_slice(&data)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error).into()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(error) => Err(error.into()),
        }
    }

    pub fn save(&self, preset: &CustomPreset) -> Result<(), T4lError> {
        let mut presets = self.try_load()?;

        if let Some(existing) = presets.iter_mut().find(|p| p.name == preset.name) {
            *existing = preset.clone();
        } else {
            presets.push(preset.clone());
        }

        self.write(&presets)
    }

    pub fn delete(&self, name: &str) -> Result<(), T4lError> {
        let mut presets = self.try_load()?;

        let Some(index) = presets.iter().position(|p| p.name == name) else {
            return Err(T4lError::PresetNotFound(name.to_string()));
        };

        presets.remove(index);

        self.write(&presets)
    }

    fn write(&self, presets: &[CustomPreset]) -> Result<(), T4lError> {
        let data = serde_json::to_vec_pretty(presets).map_err(|_| T4lError::InvalidSetting)?;

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        // A failed write must not truncate the previous preset collection.
        static WRITE_COUNTER: AtomicUsize = AtomicUsize::new(0);
        let temporary = self.path.with_extension(format!(
            "tmp-{}-{}",
            std::process::id(),
            WRITE_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let result = (|| {
            fs::write(&temporary, data)?;
            fs::rename(&temporary, &self.path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result?;

        Ok(())
    }
}

fn presets_file_path() -> PathBuf {
    config_path(
        std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
    )
}

fn config_path(xdg: Option<PathBuf>, home: Option<PathBuf>) -> PathBuf {
    xdg.filter(|path| path.is_absolute())
        .or_else(|| {
            home.filter(|path| path.is_absolute())
                .map(|path| path.join(".config"))
        })
        .unwrap_or_else(|| PathBuf::from(".config"))
        .join("tiny4linux/custom_presets.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertables::assert_err;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn test_store() -> CustomPresetStore {
        let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        CustomPresetStore::with_path(std::env::temp_dir().join(format!(
            "tiny4linux-custom-presets-test-{}-{}",
            std::process::id(),
            id
        )))
    }

    fn test_preset(name: &str) -> CustomPreset {
        CustomPreset {
            name: name.to_string(),
            pan: 0,
            tilt: 0,
            zoom: 0,
        }
    }

    #[test]
    fn load_missing_file_returns_empty() {
        let store = test_store();

        assert_eq!(store.load(), Vec::<CustomPreset>::new());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let store = test_store();
        let preset = test_preset("Desk");

        store.save(&preset).unwrap();

        assert_eq!(store.load(), vec![preset]);
    }

    #[test]
    fn save_overwrites_existing_preset() {
        let store = test_store();
        let mut preset = test_preset("Desk");
        store.save(&preset).unwrap();

        preset.pan = 180_000;
        store.save(&preset).unwrap();

        let presets = store.load();
        assert_eq!(presets.len(), 1);
        assert_eq!(presets[0], preset);
    }

    #[test]
    fn delete_removes_preset() {
        let store = test_store();
        store.save(&test_preset("Desk")).unwrap();
        store.save(&test_preset("Whiteboard")).unwrap();

        store.delete("Desk").unwrap();

        let presets = store.load();
        assert_eq!(presets.len(), 1);
        assert_eq!(presets[0].name, "Whiteboard");
    }

    #[test]
    fn delete_missing_preset_errors() {
        let store = test_store();

        assert_err!(
            store.delete("Missing"),
            "deleting a missing preset should error"
        );
    }

    #[test]
    fn load_corrupt_file_returns_empty() {
        let store = test_store();
        fs::write(&store.path, b"this is not json").unwrap();

        assert_eq!(store.load(), Vec::<CustomPreset>::new());
    }

    #[test]
    fn store_uses_user_config_directory() {
        assert_eq!(
            config_path(Some("/config".into()), Some("/home/test".into())),
            PathBuf::from("/config/tiny4linux/custom_presets.json")
        );
        assert_eq!(
            config_path(Some("relative".into()), Some("/home/test".into())),
            PathBuf::from("/home/test/.config/tiny4linux/custom_presets.json")
        );
    }

    #[test]
    fn corrupt_store_is_not_overwritten_by_save_or_delete() {
        let store = test_store();
        let corrupt = b"this is not json";
        fs::write(&store.path, corrupt).unwrap();
        assert!(store.save(&test_preset("Desk")).is_err());
        assert!(store.delete("Desk").is_err());
        assert_eq!(fs::read(&store.path).unwrap(), corrupt);
    }

    #[test]
    fn writes_valid_json() {
        let store = test_store();
        store.save(&test_preset("Desk")).unwrap();

        let raw = fs::read_to_string(&store.path).unwrap();
        let parsed: Vec<CustomPreset> = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed, store.load());
        assert!(raw.contains("\"name\""));
    }
}
