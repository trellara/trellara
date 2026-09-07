use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::{
    migrate_config_yaml, redact_config_yaml, CliError, ConfigMigration, DatasetConfig, Result,
    SourceConfig, StreamConfig, TargetConfig, CURRENT_CONFIG_VERSION,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TrellaraConfig {
    pub config_version: u16,
    pub environment: ConfigurationEnvironment,
    pub source: SourceConfig,
    pub dataset: DatasetConfig,
    pub stream: StreamConfig,
    pub target: Option<TargetConfig>,
}

#[derive(Copy, Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConfigurationEnvironment {
    #[default]
    Development,
    Production,
}

impl std::fmt::Display for ConfigurationEnvironment {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Development => formatter.write_str("development"),
            Self::Production => formatter.write_str("production"),
        }
    }
}

impl TrellaraConfig {
    pub fn from_path(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path).map_err(|source| CliError::ReadConfig {
            path: path.display().to_string(),
            source,
        })?;
        Self::from_yaml(&contents, &path.display().to_string())
    }

    pub fn from_yaml(contents: &str, label: &str) -> Result<Self> {
        let migrated = migrate_config_yaml(contents, label)?;
        let config: Self =
            serde_yaml::from_str(&migrated.yaml).map_err(|source| CliError::ParseConfig {
                path: label.to_string(),
                source,
            })?;
        if config.config_version != CURRENT_CONFIG_VERSION {
            return Err(CliError::InvalidConfig(format!(
                "config_version must be {CURRENT_CONFIG_VERSION} after migration"
            )));
        }
        Ok(config)
    }

    pub fn migrate_yaml(contents: &str, label: &str) -> Result<ConfigMigration> {
        migrate_config_yaml(contents, label)
    }

    pub fn redacted_yaml(contents: &str, label: &str) -> Result<String> {
        redact_config_yaml(contents, label)
    }
}
