use std::path::PathBuf;

use serde::Deserialize;
use trellara_stream_kafka::KafkaProductionContract;
#[cfg(feature = "local-stream")]
use trellara_stream_local::LocalDurability;

use crate::{require_non_empty, CliError, Result};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StreamConfig {
    Kafka {
        bootstrap_servers: String,
        topic: String,
        #[serde(default)]
        consumer_group: Option<String>,
        profile: KafkaProfile,
    },
    Local {
        path: PathBuf,
        #[serde(default)]
        consumer_group: Option<String>,
        #[serde(default)]
        durability: LocalStreamDurability,
    },
}

impl StreamConfig {
    pub(crate) fn validate(&self) -> Result<()> {
        match self {
            StreamConfig::Kafka {
                bootstrap_servers,
                topic,
                profile,
                ..
            } => {
                require_non_empty("stream.bootstrap_servers", bootstrap_servers)?;
                require_non_empty("stream.topic", topic)?;
                profile.validate()
            }
            StreamConfig::Local { path, .. } => {
                if path.as_os_str().is_empty() {
                    Err(CliError::InvalidConfig(
                        "stream.path must not be empty".to_string(),
                    ))
                } else {
                    Ok(())
                }
            }
        }
    }
}

#[derive(Clone, Default, Deserialize, Eq, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum KafkaProfile {
    #[default]
    Development,
    Production {
        contract: KafkaProductionContract,
    },
}

impl KafkaProfile {
    pub(crate) fn validate(&self) -> Result<()> {
        match self {
            Self::Development => Ok(()),
            Self::Production { contract } => contract
                .validate()
                .map_err(|error| CliError::InvalidConfig(error.to_string())),
        }
    }

    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Production { .. } => "production",
        }
    }
}

impl std::fmt::Debug for KafkaProfile {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Development => formatter.write_str("Development"),
            Self::Production { contract } => formatter
                .debug_struct("Production")
                .field("contract_version", &contract.contract_version)
                .field("replication_factor", &contract.replication_factor)
                .field("min_insync_replicas", &contract.min_insync_replicas)
                .field("tls", &"<configured>")
                .field("credentials", &"<redacted>")
                .finish(),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LocalStreamDurability {
    #[default]
    Fsync,
    Buffered,
}

#[cfg(feature = "local-stream")]
impl From<LocalStreamDurability> for LocalDurability {
    fn from(durability: LocalStreamDurability) -> Self {
        match durability {
            LocalStreamDurability::Fsync => Self::Fsync,
            LocalStreamDurability::Buffered => Self::Buffered,
        }
    }
}
