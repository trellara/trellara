use serde::{Deserialize, Serialize};

use crate::rng::DeterministicRng;
use crate::snapshot_state::SnapshotSimState;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotFailurePoint {
    SourceCrashDuringTableCopy,
    RelayCrashDuringTableCopy,
    TargetCrashDuringTableCopy,
    DuplicateCopyAttempt,
    DdlDuringTableCopy,
    HandoffRecordedBeforeStreamStart,
}

impl SnapshotFailurePoint {
    pub const ALL: [Self; 6] = [
        Self::SourceCrashDuringTableCopy,
        Self::RelayCrashDuringTableCopy,
        Self::TargetCrashDuringTableCopy,
        Self::DuplicateCopyAttempt,
        Self::DdlDuringTableCopy,
        Self::HandoffRecordedBeforeStreamStart,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::SourceCrashDuringTableCopy => "snapshot_source_crash_during_table_copy",
            Self::RelayCrashDuringTableCopy => "snapshot_relay_crash_during_table_copy",
            Self::TargetCrashDuringTableCopy => "snapshot_target_crash_during_table_copy",
            Self::DuplicateCopyAttempt => "snapshot_duplicate_copy_attempt",
            Self::DdlDuringTableCopy => "snapshot_ddl_during_table_copy",
            Self::HandoffRecordedBeforeStreamStart => {
                "snapshot_handoff_recorded_before_stream_start"
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SnapshotSimulationConfig {
    pub seed: u64,
    pub failure_point: SnapshotFailurePoint,
    pub table_count: usize,
    pub writes_after_snapshot: usize,
}

impl SnapshotSimulationConfig {
    pub fn new(seed: u64, failure_point: SnapshotFailurePoint) -> Self {
        Self {
            seed,
            failure_point,
            table_count: 4,
            writes_after_snapshot: 6,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SnapshotSimulationReport {
    pub seed: u64,
    pub failure_point: SnapshotFailurePoint,
    pub table_count: usize,
    pub copied_tables: usize,
    pub writes_after_snapshot: usize,
    pub stream_replayed_transactions: usize,
    pub verification_matched: bool,
    pub handoff_recorded: bool,
    pub stream_started: bool,
    pub contract_refreshed: bool,
    pub passed: bool,
    pub injected_failure: Option<String>,
    pub steps: Vec<SnapshotSimulationStep>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SnapshotSimulationStep {
    pub relation: Option<String>,
    pub action: SnapshotSimulationAction,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotSimulationAction {
    SlotCreated,
    SnapshotExported,
    TableCopyStarted,
    SourceCrashedDuringTableCopy,
    TableCopyCrashed,
    SnapshotMarkedFailedRecoverable,
    SchemaDriftDetected,
    HandoffWithheld,
    ContractRefreshed,
    TableCopyRetried,
    TableCopyCompleted,
    DuplicateTableCopyAttempted,
    DuplicateTableCopySkipped,
    HandoffRecorded,
    RelayCrashedBeforeStreamStart,
    StreamStarted,
    PostSnapshotWriteReplayed,
    VerifiedConverged,
}

pub fn run_snapshot_simulation(config: SnapshotSimulationConfig) -> SnapshotSimulationReport {
    let tables = generated_snapshot_tables(config.seed, config.table_count);
    let mut state = SnapshotSimState::new(config, tables);
    state.run();
    state.report()
}

pub fn run_default_snapshot_suite(seed: u64) -> Vec<SnapshotSimulationReport> {
    SnapshotFailurePoint::ALL
        .into_iter()
        .enumerate()
        .map(|(index, failure_point)| {
            run_snapshot_simulation(SnapshotSimulationConfig::new(
                seed.wrapping_add(100 + index as u64),
                failure_point,
            ))
        })
        .collect()
}

fn generated_snapshot_tables(seed: u64, count: usize) -> Vec<String> {
    let mut rng = DeterministicRng::new(seed);
    (0..count)
        .map(|index| format!("public.snapshot_{:02}_{:08x}", index + 1, rng.next() as u32))
        .collect()
}
