use std::{fmt::Display, fs, io};

use serde::{Deserialize, Serialize};

fn default_favorites() -> Vec<String> {
    [
        "BTCUSDT", "ETHUSDT", "LINKUSDT", "UNIUSDT", "ARBUSDT", "SYNUSDT", "OPUSDT",
    ]
    .into_iter()
    .map(ToOwned::to_owned)
    .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub(crate) api_key: String,
    pub(crate) api_secret_key: String,
    #[serde(default = "default_favorites")]
    pub(crate) watchlist_favorites: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_secret_key: String::new(),
            watchlist_favorites: default_favorites(),
        }
    }
}

#[derive(Debug)]
pub enum LoadError {
    IO(io::Error),
    Format(String),
}

#[derive(Debug)]
pub enum SaveError {
    File(io::Error),
    Write(io::Error),
}

impl Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::File(err) | SaveError::Write(err) => err,
        }
        .fmt(f)
    }
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

    pub(crate) fn load() -> Result<Option<Config>, LoadError> {
        let contents = match fs::read_to_string(Self::path()) {
            Ok(contents) => Ok(contents),
            Err(err) => {
                if matches!(err.kind(), io::ErrorKind::NotFound) {
                    return Ok(None);
                }
                Err(LoadError::IO(err))
            }
        }?;

        serde_json::from_str(&contents).map_err(|err| LoadError::Format(err.to_string()))
    }

    pub(crate) fn save(&self) -> Result<(), SaveError> {
        let json = serde_json::to_string_pretty(&self).expect("config serializer is valid");
        let path = Self::path();
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(SaveError::File)?;
        }
        fs::write(path, json.as_bytes()).map_err(SaveError::Write)
    }

    pub(crate) fn credentials(&self) -> (&str, &str) {
        (&self.api_key, &self.api_secret_key)
    }

    fn crendentials_empty(&self) -> bool {
        self.api_key.is_empty() || self.api_secret_key.is_empty()
    }

    pub(crate) fn complete(&self) -> bool {
        !self.crendentials_empty()
    }
}

#[cfg(target_arch = "wasm32")]
impl Config {
    fn storage() -> Option<web_sys::Storage> {
        let window = web_sys::window()?;

        window.local_storage().ok()?
    }

    pub(crate) fn load() -> Result<Option<Config>, LoadError> {
        let storage = Self::storage().ok_or(LoadError::File)?;

        let contents = storage
            .get_item("state")
            .map_err(LoadError::File)?
            .ok_or(LoadError::File)?;

        Some(serde_json::from_str(&contents).map_err(|_| LoadError::Format))
    }

    pub(crate) fn save(self) -> Result<(), SaveError> {
        let storage = Self::storage().ok_or(SaveError::File)?;

        let json = serde_json::to_string_pretty(&self).expect("config serializer is valid");

        storage.set_item("state", &json).map_err(SaveError::Write)?;

        Ok(())
    }
}
