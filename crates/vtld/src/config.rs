use std::path::Path;

use serde::Deserialize;

use crate::error::{Error, Result};

#[derive(Debug, Clone, Deserialize)]
pub struct UserConfig {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub listen: ListenConfig,
    pub store: StoreConfig,
    pub library: LibraryConfig,
    #[serde(default)]
    pub iscsi: IscsiConfig,
    #[serde(default)]
    pub users: Vec<UserConfig>,
    /// Simulation speed factor (1.0 = realistic, higher = faster).
    /// Defaults to 1.0 (real-time). Set to a very large value for instant.
    #[serde(default = "default_simulation_speed")]
    pub simulation_speed: f64,
}

fn default_simulation_speed() -> f64 {
    1.0
}

#[derive(Debug, Deserialize)]
pub struct IscsiConfig {
    #[serde(default = "default_iscsi_port")]
    pub port: u16,
    #[serde(default = "default_iscsi_iqn")]
    pub iqn: String,
}

impl Default for IscsiConfig {
    fn default() -> Self {
        Self {
            port: default_iscsi_port(),
            iqn: default_iscsi_iqn(),
        }
    }
}

fn default_iscsi_port() -> u16 {
    3260
}

fn default_iscsi_iqn() -> String {
    "iqn.2024-01.com.quantumvtl:vtl".to_owned()
}

#[derive(Debug, Deserialize)]
pub struct ListenConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_admin_port")]
    pub admin_port: u16,
}

fn default_host() -> String {
    "0.0.0.0".to_owned()
}

fn default_admin_port() -> u16 {
    8081
}

#[derive(Debug, Deserialize)]
pub struct StoreConfig {
    #[serde(default = "default_db_path")]
    pub path: String,
}

fn default_db_path() -> String {
    "vtld.redb".to_owned()
}

#[derive(Debug, Deserialize)]
pub struct LibraryConfig {
    pub model: String,
    pub serial: String,
    pub data_dir: String,
    pub drives: usize,
    pub slots: usize,
    #[serde(default)]
    pub media: Vec<MediaConfig>,
}

#[derive(Debug, Deserialize)]
pub struct MediaConfig {
    pub barcode: String,
}

pub fn load_config(path: impl AsRef<Path>) -> Result<Config> {
    let contents = std::fs::read_to_string(path.as_ref())
        .map_err(|e| Error::Config(format!("failed to read config file: {e}")))?;
    let config: Config = serde_yaml::from_str(&contents)
        .map_err(|e| Error::Config(format!("failed to parse config: {e}")))?;
    Ok(config)
}
