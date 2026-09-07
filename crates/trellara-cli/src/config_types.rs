use serde::Deserialize;

use crate::{CliError, PartitionConfig, Result, SensitiveString, TableConfig};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct DatasetConfig {
    pub id: String,
    pub mode: DatasetMode,
    #[serde(default)]
    pub unknown_table_policy: UnknownTablePolicy,
    pub tables: Vec<TableConfig>,
    #[serde(default)]
    pub partition: Option<PartitionConfig>,
    #[serde(default)]
    pub strict_chunking: Option<StrictChunkConfig>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct StrictChunkConfig {
    pub max_changes_per_chunk: u32,
}

#[derive(Copy, Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UnknownTablePolicy {
    #[default]
    Reject,
    AllowCompatible,
}

impl std::fmt::Display for UnknownTablePolicy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnknownTablePolicy::Reject => formatter.write_str("reject"),
            UnknownTablePolicy::AllowCompatible => formatter.write_str("allow_compatible"),
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DatasetMode {
    StrictTransactionOrder,
    PartitionedScaleMode,
}

impl std::fmt::Display for DatasetMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatasetMode::StrictTransactionOrder => formatter.write_str("strict_transaction_order"),
            DatasetMode::PartitionedScaleMode => formatter.write_str("partitioned_scale_mode"),
        }
    }
}

impl DatasetMode {
    pub(crate) fn validate(&self, partition: Option<&PartitionConfig>) -> Result<()> {
        match self {
            DatasetMode::StrictTransactionOrder => Ok(()),
            DatasetMode::PartitionedScaleMode => {
                let partition = partition.ok_or_else(|| {
                    CliError::InvalidConfig(
                        "dataset.partition is required for partitioned_scale_mode".to_string(),
                    )
                })?;
                if partition.partition_count == 0 {
                    return Err(CliError::InvalidConfig(
                        "dataset.partition.partition_count must be greater than zero".to_string(),
                    ));
                }
                require_non_empty("dataset.partition.key_column", &partition.key_column)
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TargetConfig {
    pub database_url: SensitiveString,
}

pub(crate) fn require_non_empty(field: &'static str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        Err(CliError::InvalidConfig(format!(
            "{field} must not be empty"
        )))
    } else {
        Ok(())
    }
}

pub(crate) fn reject_surrounding_whitespace(field: &'static str, value: &str) -> Result<()> {
    if value != value.trim() {
        Err(CliError::InvalidConfig(format!(
            "{field} must not contain surrounding whitespace"
        )))
    } else {
        Ok(())
    }
}

pub(crate) fn default_primary_key() -> String {
    "id".to_string()
}
