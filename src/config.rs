// src/config.rs
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub storage_location: Option<PathBuf>,
}

impl AppConfig {
    pub fn load() -> Self {
        let config_path = default_config_path();
        let file = File::open(&config_path);
        match file {
            Ok(file) => {
                let reader = BufReader::new(file);
                serde_json::from_reader(reader).unwrap_or_else(|_| Self::default())
            }
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let config_path = default_config_path();
        if !config_path.exists() {
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)?;
            }
        }

        let file = File::create(&config_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)?;
        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            storage_location: None,
        }
    }
}

fn default_config_path() -> PathBuf {
    let mut config_path = match dirs::config_dir() {
        Some(dir) => dir,
        None => PathBuf::from("."),
    };
    config_path.push("time_tracker");
    config_path.push("config.json");
    config_path
}
