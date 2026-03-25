use serde::Deserialize;
use std::{env, fs, path::PathBuf};

use crate::error::{Result, VelnError};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub zfs_pool: String,
    pub vm_root: String,
}

impl Config {
    /// # Errors
    /// Returns `VelnError::Config` if the file cannot be read or contains invalid TOML.
    pub fn load() -> Result<Self> {
        let path = Self::path();

        let content = fs::read_to_string(&path)
            .map_err(|e| VelnError::Config(format!("Failed to read {}: {e}", path.display())))?;

        toml::from_str(&content)
            .map_err(|e| VelnError::Config(format!("Invalid config: {e}")))
    }

    /// Config file path: `VELN_CONFIG` env var, or `/usr/local/etc/veln/config.toml`.
    fn path() -> PathBuf {
        env::var("VELN_CONFIG")
            .map_or_else(|_| PathBuf::from("/usr/local/etc/veln/config.toml"), PathBuf::from)
    }
}
