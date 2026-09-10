use crate::liveness::LivenessMode;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub match_threshold: f32,
    pub liveness_mode: LivenessMode,
    pub camera_index: usize,
    pub scan_timeout_seconds: u64,
    pub auto_retry: bool,
    pub enable_notification: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            match_threshold: 0.65,
            liveness_mode: LivenessMode::Light,
            camera_index: 0,
            scan_timeout_seconds: 5,
            auto_retry: true,
            enable_notification: true,
        }
    }
}

pub struct ConfigManager;

impl ConfigManager {
    pub fn get_app_dir() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "mukalujauh", "mukalujauh") {
            proj_dirs.config_dir().to_path_buf()
        } else {
            PathBuf::from(".mukalujauh")
        }
    }

    pub fn get_config_path() -> PathBuf {
        Self::get_app_dir().join("config.toml")
    }

    pub fn get_database_path() -> PathBuf {
        Self::get_app_dir().join("identities.enc")
    }

    pub fn load_config() -> AppConfig {
        let config_path = Self::get_config_path();
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = toml::from_str(&content) {
                return config;
            }
        }
        AppConfig::default()
    }

    pub fn save_config(config: &AppConfig) -> Result<()> {
        let dir = Self::get_app_dir();
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create config dir {:?}", dir))?;
        let content = toml::to_string_pretty(config)?;
        fs::write(Self::get_config_path(), content)?;
        Ok(())
    }
}
