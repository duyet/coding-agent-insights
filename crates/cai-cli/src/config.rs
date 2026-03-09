//! Configuration management for CAI CLI
//!
//! Handles loading configuration from ~/.cai/config.toml or XDG config directories.

use cai_core::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Re-export the config crate types
use config::{Config, Environment, File};

/// CAI configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaiConfig {
    /// Storage configuration
    #[serde(default)]
    pub storage: StorageConfig,
    /// Output configuration
    #[serde(default)]
    pub output: OutputConfig,
}

impl Default for CaiConfig {
    fn default() -> Self {
        Self {
            storage: StorageConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

/// Storage backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Storage type: "memory" or "sqlite"
    #[serde(default = "default_storage_type")]
    pub r#type: String,

    /// Database path (for sqlite)
    #[serde(default)]
    pub path: Option<String>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            r#type: default_storage_type(),
            path: None,
        }
    }
}

fn default_storage_type() -> String {
    "memory".to_string()
}

/// Output format configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Default output format
    #[serde(default = "default_output_format")]
    pub format: String,

    /// Maximum rows to display
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: default_output_format(),
            max_rows: default_max_rows(),
        }
    }
}

fn default_output_format() -> String {
    "table".to_string()
}

fn default_max_rows() -> usize {
    100
}

/// Get the CAI config directory
///
/// Checks for config in:
/// 1. ~/.cai/config.toml
/// 2. $XDG_CONFIG_HOME/cai/config.toml (defaults to ~/.config/cai/config.toml)
pub fn get_config_dir() -> PathBuf {
    // Check ~/.cai first
    let home_dir = dirs::home_dir().expect("Unable to determine home directory");
    let cai_dir = home_dir.join(".cai");

    if cai_dir.exists() {
        return cai_dir;
    }

    // Fall back to XDG config directory
    if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("cai")
    } else {
        cai_dir
    }
}

/// Get the configuration file path
pub fn get_config_file() -> PathBuf {
    get_config_dir().join("config.toml")
}

/// Load configuration from file
///
/// Returns default config if file doesn't exist or can't be parsed
pub fn load_config() -> CaiConfig {
    let config_path = get_config_file();

    if !config_path.exists() {
        tracing::debug!("No config file at {:?}, using defaults", config_path);
        return CaiConfig::default();
    }

    let config_str = config_path.to_string_lossy().to_string();

    let built_settings = match Config::builder()
        .add_source(File::with_name(&config_str))
        .add_source(Environment::with_prefix("CAI").separator("_"))
        .build()
    {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(
                "Failed to load config file {:?}: {:?}, using defaults",
                config_path,
                e
            );
            return CaiConfig::default();
        }
    };

    match built_settings.try_deserialize::<CaiConfig>() {
        Ok(config) => {
            tracing::debug!("Loaded config from {:?}", config_path);
            config
        }
        Err(e) => {
            tracing::warn!(
                "Failed to parse config file {:?}: {:?}, using defaults",
                config_path,
                e
            );
            CaiConfig::default()
        }
    }
}

/// Save configuration to file
#[allow(dead_code)]
pub fn save_config(config: &CaiConfig) -> Result<()> {
    let config_path = get_config_file();

    // Ensure config directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            cai_core::Error::Message(format!("Failed to create config directory: {}", e))
        })?;
    }

    // Serialize to TOML
    let toml_string = toml::to_string_pretty(config)
        .map_err(|e| cai_core::Error::Message(format!("Failed to serialize config: {}", e)))?;

    // Write to file
    std::fs::write(&config_path, toml_string)
        .map_err(|e| cai_core::Error::Message(format!("Failed to write config file: {}", e)))?;

    tracing::info!("Config saved to {:?}", config_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CaiConfig::default();
        assert_eq!(config.storage.r#type, "memory");
        assert_eq!(config.output.format, "table");
        assert_eq!(config.output.max_rows, 100);
    }

    #[test]
    fn test_config_dir() {
        let config_dir = get_config_dir();
        assert!(config_dir.ends_with("cai") || config_dir.ends_with(".cai"));
    }

    #[test]
    fn test_config_file() {
        let config_file = get_config_file();
        assert!(config_file.ends_with("config.toml"));
    }
}
