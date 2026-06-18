use std::{fs, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub default_hash: Option<String>,
}

pub fn load() -> Config {
    let Some(mut path) = dirs::config_dir() else {
        return Config::default();
    };
    path.push("mpct");
    path.push("config.toml");
    load_from_path(path).unwrap_or_default()
}

fn load_from_path(path: PathBuf) -> Option<Config> {
    let contents = fs::read_to_string(path).ok()?;
    toml::from_str(&contents).ok()
}
