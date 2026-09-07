use std::fs;
use std::path::Path;

use serde_yaml::{Mapping, Value};

use crate::{CliError, Result};

pub const CURRENT_CONFIG_VERSION: u16 = 2;
const LEGACY_CONFIG_VERSION: u16 = 1;

#[derive(Clone, Eq, PartialEq)]
pub struct ConfigMigration {
    pub from_version: u16,
    pub to_version: u16,
    pub migrated: bool,
    pub yaml: String,
}

impl std::fmt::Debug for ConfigMigration {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ConfigMigration")
            .field("from_version", &self.from_version)
            .field("to_version", &self.to_version)
            .field("migrated", &self.migrated)
            .field("yaml", &"<redacted>")
            .finish()
    }
}

pub fn migrate_config_path(path: &Path) -> Result<ConfigMigration> {
    let label = path.display().to_string();
    let contents = fs::read_to_string(path).map_err(|source| CliError::ReadConfig {
        path: label.clone(),
        source,
    })?;
    migrate_config_yaml(&contents, &label)
}

pub fn migrate_config_yaml(contents: &str, label: &str) -> Result<ConfigMigration> {
    let mut document: Value =
        serde_yaml::from_str(contents).map_err(|source| CliError::ParseConfig {
            path: label.to_string(),
            source,
        })?;
    let from_version = document_version(&document)?;
    let mut version = from_version;
    while version < CURRENT_CONFIG_VERSION {
        match version {
            LEGACY_CONFIG_VERSION => migrate_v1_to_v2(&mut document)?,
            _ => return Err(unsupported_version(version)),
        }
        version += 1;
    }
    if version != CURRENT_CONFIG_VERSION {
        return Err(unsupported_version(version));
    }
    let yaml = serde_yaml::to_string(&document).map_err(|source| CliError::ParseConfig {
        path: label.to_string(),
        source,
    })?;
    Ok(ConfigMigration {
        from_version,
        to_version: version,
        migrated: from_version != version,
        yaml,
    })
}

fn document_version(document: &Value) -> Result<u16> {
    let mapping = document.as_mapping().ok_or_else(|| {
        CliError::InvalidConfig("configuration root must be a YAML mapping".to_string())
    })?;
    let Some(value) = mapping.get(key("config_version")) else {
        return Ok(LEGACY_CONFIG_VERSION);
    };
    let raw = value.as_u64().ok_or_else(|| {
        CliError::InvalidConfig("config_version must be a positive integer".to_string())
    })?;
    u16::try_from(raw)
        .ok()
        .filter(|version| *version > 0)
        .ok_or_else(|| CliError::InvalidConfig("config_version is out of range".to_string()))
}

fn migrate_v1_to_v2(document: &mut Value) -> Result<()> {
    let existing = document.as_mapping_mut().ok_or_else(|| {
        CliError::InvalidConfig("configuration root must be a YAML mapping".to_string())
    })?;
    let mut migrated = Mapping::new();
    migrated.insert(key("config_version"), Value::from(CURRENT_CONFIG_VERSION));
    migrated.insert(key("environment"), Value::from("development"));
    for (name, mut value) in std::mem::take(existing) {
        if name == key("config_version") || name == key("environment") {
            continue;
        }
        if name == key("stream") {
            add_development_kafka_profile(&mut value)?;
        }
        migrated.insert(name, value);
    }
    *existing = migrated;
    Ok(())
}

fn add_development_kafka_profile(stream: &mut Value) -> Result<()> {
    let mapping = stream
        .as_mapping_mut()
        .ok_or_else(|| CliError::InvalidConfig("stream must be a YAML mapping".to_string()))?;
    if mapping.get(key("kind")).and_then(Value::as_str) == Some("kafka")
        && !mapping.contains_key(key("profile"))
    {
        let mut profile = Mapping::new();
        profile.insert(key("kind"), Value::from("development"));
        mapping.insert(key("profile"), Value::Mapping(profile));
    }
    Ok(())
}

fn unsupported_version(version: u16) -> CliError {
    CliError::InvalidConfig(format!(
        "unsupported config_version {version}; this binary supports {CURRENT_CONFIG_VERSION}"
    ))
}

fn key(name: &str) -> Value {
    Value::String(name.to_string())
}
