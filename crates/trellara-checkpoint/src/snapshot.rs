use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{CheckpointError, Result};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotRunState {
    Planned,
    SlotCreated,
    SnapshotExported,
    CopyingTable,
    CopyComplete,
    StreamHandoffReady,
    Streaming,
    Verified,
    FailedRecoverable,
}

impl std::fmt::Display for SnapshotRunState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnapshotRunState::Planned => formatter.write_str("planned"),
            SnapshotRunState::SlotCreated => formatter.write_str("slot_created"),
            SnapshotRunState::SnapshotExported => formatter.write_str("snapshot_exported"),
            SnapshotRunState::CopyingTable => formatter.write_str("copying_table"),
            SnapshotRunState::CopyComplete => formatter.write_str("copy_complete"),
            SnapshotRunState::StreamHandoffReady => formatter.write_str("stream_handoff_ready"),
            SnapshotRunState::Streaming => formatter.write_str("streaming"),
            SnapshotRunState::Verified => formatter.write_str("verified"),
            SnapshotRunState::FailedRecoverable => formatter.write_str("failed_recoverable"),
        }
    }
}

impl FromStr for SnapshotRunState {
    type Err = CheckpointError;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "planned" => Ok(Self::Planned),
            "slot_created" => Ok(Self::SlotCreated),
            "snapshot_exported" => Ok(Self::SnapshotExported),
            "copying_table" => Ok(Self::CopyingTable),
            "copy_complete" => Ok(Self::CopyComplete),
            "stream_handoff_ready" => Ok(Self::StreamHandoffReady),
            "streaming" => Ok(Self::Streaming),
            "verified" => Ok(Self::Verified),
            "failed_recoverable" => Ok(Self::FailedRecoverable),
            _ => Err(CheckpointError::Store(format!(
                "unknown snapshot run state {value:?}"
            ))),
        }
    }
}

impl SnapshotRunState {
    pub fn can_start_as(self) -> bool {
        matches!(self, Self::Planned | Self::SlotCreated)
    }

    pub fn can_transition_to(self, next: Self) -> bool {
        if self == next {
            return true;
        }

        match self {
            Self::Planned => matches!(next, Self::SlotCreated),
            Self::SlotCreated => matches!(
                next,
                Self::SnapshotExported | Self::CopyingTable | Self::FailedRecoverable
            ),
            Self::SnapshotExported => {
                matches!(next, Self::CopyingTable | Self::FailedRecoverable)
            }
            Self::CopyingTable => matches!(
                next,
                Self::CopyingTable | Self::CopyComplete | Self::FailedRecoverable
            ),
            Self::CopyComplete => {
                matches!(next, Self::StreamHandoffReady | Self::FailedRecoverable)
            }
            Self::StreamHandoffReady => {
                matches!(
                    next,
                    Self::Streaming | Self::Verified | Self::FailedRecoverable
                )
            }
            Self::Streaming => matches!(next, Self::Verified | Self::FailedRecoverable),
            Self::Verified => matches!(next, Self::Verified | Self::FailedRecoverable),
            Self::FailedRecoverable => matches!(
                next,
                Self::Planned | Self::SlotCreated | Self::SnapshotExported | Self::CopyingTable
            ),
        }
    }

    pub fn validate_transition(self, next: Self) -> Result<()> {
        if self.can_transition_to(next) {
            Ok(())
        } else {
            Err(CheckpointError::Store(format!(
                "invalid snapshot run transition from {self} to {next}"
            )))
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SnapshotRun {
    pub source_id: String,
    pub dataset_id: String,
    pub run_id: String,
    pub state: SnapshotRunState,
    pub slot_name: String,
    pub consistent_lsn: Option<String>,
    pub current_relation: Option<String>,
    pub copied_rows: i64,
    pub failure_reason: Option<String>,
    pub started_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SnapshotTableProgress {
    pub source_id: String,
    pub dataset_id: String,
    pub run_id: String,
    pub relation: String,
    pub state: SnapshotRunState,
    pub copied_rows: i64,
    pub watermark_lsn: Option<String>,
    pub updated_at: String,
}
