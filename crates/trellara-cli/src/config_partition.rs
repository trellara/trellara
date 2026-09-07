use serde::Deserialize;
use trellara_protocol::{
    PartitionKeyChangePolicy as ProtocolPartitionKeyChangePolicy,
    PartitionNullKeyPolicy as ProtocolPartitionNullKeyPolicy,
};

use crate::ContractSeverity;

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PartitionNullKeyPolicy {
    Quarantine,
    RouteToDeadLetterPartition,
    RouteToSingletonPartition,
    DeriveFromPrimaryKey,
}

impl std::fmt::Display for PartitionNullKeyPolicy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            PartitionNullKeyPolicy::Quarantine => "quarantine",
            PartitionNullKeyPolicy::RouteToDeadLetterPartition => "route_to_dead_letter_partition",
            PartitionNullKeyPolicy::RouteToSingletonPartition => "route_to_singleton_partition",
            PartitionNullKeyPolicy::DeriveFromPrimaryKey => "derive_from_primary_key",
        })
    }
}

impl PartitionNullKeyPolicy {
    pub(crate) fn to_protocol_policy(self) -> ProtocolPartitionNullKeyPolicy {
        match self {
            PartitionNullKeyPolicy::Quarantine => ProtocolPartitionNullKeyPolicy::Quarantine,
            PartitionNullKeyPolicy::RouteToDeadLetterPartition => {
                ProtocolPartitionNullKeyPolicy::RouteToDeadLetterPartition
            }
            PartitionNullKeyPolicy::RouteToSingletonPartition => {
                ProtocolPartitionNullKeyPolicy::RouteToSingletonPartition
            }
            PartitionNullKeyPolicy::DeriveFromPrimaryKey => {
                ProtocolPartitionNullKeyPolicy::DeriveFromPrimaryKey
            }
        }
    }
}

fn default_partition_null_key_policy() -> PartitionNullKeyPolicy {
    PartitionNullKeyPolicy::Quarantine
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PartitionKeyChangePolicy {
    Quarantine,
    EmitMove,
    DualWriteWindow,
    Forbid,
}

impl PartitionKeyChangePolicy {
    pub(crate) fn contract_severity(&self) -> ContractSeverity {
        match self {
            PartitionKeyChangePolicy::DualWriteWindow => ContractSeverity::Warning,
            PartitionKeyChangePolicy::Quarantine
            | PartitionKeyChangePolicy::EmitMove
            | PartitionKeyChangePolicy::Forbid => ContractSeverity::Info,
        }
    }

    pub(crate) fn contract_message(&self) -> String {
        match self {
            PartitionKeyChangePolicy::Quarantine => {
                "partition key changes use policy quarantine".to_string()
            }
            PartitionKeyChangePolicy::EmitMove => {
                "partition key changes use policy emit_move and must stay under one manifest and commit marker barrier"
                    .to_string()
            }
            PartitionKeyChangePolicy::DualWriteWindow => {
                "partition key changes use policy dual_write_window, which can expose duplicate ownership during the configured migration window".to_string()
            }
            PartitionKeyChangePolicy::Forbid => {
                "partition key changes use policy forbid and should be rejected before CDC"
                    .to_string()
            }
        }
    }
}

impl std::fmt::Display for PartitionKeyChangePolicy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            PartitionKeyChangePolicy::Quarantine => "quarantine",
            PartitionKeyChangePolicy::EmitMove => "emit_move",
            PartitionKeyChangePolicy::DualWriteWindow => "dual_write_window",
            PartitionKeyChangePolicy::Forbid => "forbid",
        })
    }
}

impl PartitionKeyChangePolicy {
    pub(crate) fn to_protocol_policy(self) -> ProtocolPartitionKeyChangePolicy {
        match self {
            PartitionKeyChangePolicy::Quarantine => ProtocolPartitionKeyChangePolicy::Quarantine,
            PartitionKeyChangePolicy::EmitMove => ProtocolPartitionKeyChangePolicy::EmitMove,
            PartitionKeyChangePolicy::DualWriteWindow => {
                ProtocolPartitionKeyChangePolicy::DualWriteWindow
            }
            PartitionKeyChangePolicy::Forbid => ProtocolPartitionKeyChangePolicy::Forbid,
        }
    }
}

fn default_partition_key_change_policy() -> PartitionKeyChangePolicy {
    PartitionKeyChangePolicy::Quarantine
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct PartitionConfig {
    pub partition_count: u32,
    pub key_column: String,
    #[serde(default = "default_partition_null_key_policy")]
    pub null_key_policy: PartitionNullKeyPolicy,
    #[serde(default = "default_partition_key_change_policy")]
    pub key_change_policy: PartitionKeyChangePolicy,
}
