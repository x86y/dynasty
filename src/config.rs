use std::fs;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub api_key: String,
    pub api_secret_key: String,
}

#[derive(Debug, Clone)]
pub enum LoadError {
    File,
    Format,
}

#[derive(Debug, Clone)]
pub enum SaveError {
    File,
    Write,
    Format,
}

#[cfg(not(target_arch = "wasm32"))]
impl Config {
    fn path() -> std::path::PathBuf {
        let mut path = if let Some(project_dirs) =
            directories_next::ProjectDirs::from("rs", "x86y", "Dynasty")
        {
            project_dirs.config_dir().into()
        } else {
            std::env::current_dir().unwrap_or_default()
        };
        path.push("config.json");
        path
    }

    pub fn load() -> Result<Config, LoadError> {
        let contents = fs::read_to_string(Self::path()).map_err(|_| LoadError::File)?;
        serde_json::from_str(&contents).map_err(|_| LoadError::Format)
    }

    pub fn save(&self) -> Result<(), SaveError> {
        let json = serde_json::to_string_pretty(&self).map_err(|_| SaveError::Format)?;
        let path = Self::path();
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|_| SaveError::File)?;
        }
        fs::write(path, json.as_bytes()).map_err(|_| SaveError::Write)
    }
}

#[cfg(target_arch = "wasm32")]
impl Config {
    fn storage() -> Option<web_sys::Storage> {
        let window = web_sys::window()?;

        window.local_storage().ok()?
    }

    fn load() -> Result<Config, LoadError> {
        let storage = Self::storage().ok_or(LoadError::File)?;

        let contents = storage
            .get_item("state")
            .map_err(|_| LoadError::File)?
            .ok_or(LoadError::File)?;

        serde_json::from_str(&contents).map_err(|_| LoadError::Format)
    }

    fn save(self) -> Result<(), SaveError> {
        let storage = Self::storage().ok_or(SaveError::File)?;

        let json = serde_json::to_string_pretty(&self).map_err(|_| SaveError::Format)?;

        storage
            .set_item("state", &json)
            .map_err(|_| SaveError::Write)?;

        Ok(())
    }
}
