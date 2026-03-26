use serde::Deserialize;
use std::{collections::HashMap, env, fs, path::PathBuf};

use crate::error::{Result, VelnError};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub zfs_pool: String,
    pub vm_root: String,
    #[serde(default)]
    pub api: ApiConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
    #[serde(default = "default_auth_enabled")]
    pub auth_enabled: bool,
    #[serde(default)]
    pub keys: HashMap<String, ApiKey>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            auth_enabled: default_auth_enabled(),
            keys: HashMap::new(),
        }
    }
}

fn default_auth_enabled() -> bool {
    true
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiKey {
    pub name: String,
    #[serde(default = "default_role")]
    pub role: String,
}

impl ApiKey {
    pub fn new(name: &str, role: &str) -> Self {
        Self {
            name: name.to_string(),
            role: role.to_string(),
        }
    }
}

fn default_role() -> String {
    "admin".to_string()
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

    /// Validate an API key and return the info if valid
    pub fn validate_api_key(&self, key: &str) -> Option<ApiKey> {
        if !self.api.auth_enabled {
            // If auth is disabled, treat any key as valid admin
            return Some(ApiKey::new("anonymous", "admin"));
        }
        self.api.keys.get(key).cloned()
    }
}
