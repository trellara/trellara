use serde::{Deserialize, Serialize};

use crate::strict_chunk_state::StrictChunkSimState;
use crate::transaction::generated_transactions;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrictChunkFailurePoint {
    RelayCrashAfterChunksBeforeManifest,
    RelayCrashAfterManifestBeforeSourceAck,
    ManifestArrivesWithMissingChunk,
    ManifestArrivesBeforeChunks,
    TargetCrashAfterStagingBeforeCommit,
}

impl StrictChunkFailurePoint {
    pub const ALL: [Self; 5] = [
        Self::RelayCrashAfterChunksBeforeManifest,
        Self::RelayCrashAfterManifestBeforeSourceAck,
        Self::ManifestArrivesWithMissingChunk,
        Self::ManifestArrivesBeforeChunks,
        Self::TargetCrashAfterStagingBeforeCommit,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::RelayCrashAfterChunksBeforeManifest => {
                "strict_chunk_relay_crash_after_chunks_before_manifest"
            }
            Self::RelayCrashAfterManifestBeforeSourceAck => {
                "strict_chunk_relay_crash_after_manifest_before_source_ack"
            }
            Self::ManifestArrivesWithMissingChunk => {
                "strict_chunk_manifest_arrives_with_missing_chunk"
            }
            Self::ManifestArrivesBeforeChunks => "strict_chunk_manifest_arrives_before_chunks",
            Self::TargetCrashAfterStagingBeforeCommit => {
                "strict_chunk_target_crash_after_staging_before_commit"
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StrictChunkSimulationConfig {
    pub seed: u64,
    pub failure_point: StrictChunkFailurePoint,
    pub chunk_count: usize,
}

impl StrictChunkSimulationConfig {
    pub fn new(seed: u64, failure_point: StrictChunkFailurePoint) -> Self {
        Self {
            seed,
            failure_point,
            chunk_count: 4,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StrictChunkSimulationReport {
    pub seed: u64,
    pub failure_point: StrictChunkFailurePoint,
    pub transaction_id: String,
    pub commit_lsn: u64,
    pub chunk_count: usize,
    pub chunks_published: usize,
    pub duplicate_chunks: usize,
    pub manifest_published: bool,
    pub source_acknowledged_lsn: Option<u64>,
    pub target_applied_lsn: Option<u64>,
    pub applied_transactions: usize,
    pub passed: bool,
    pub injected_failure: Option<String>,
    pub steps: Vec<StrictChunkSimulationStep>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StrictChunkSimulationStep {
    pub chunk_id: Option<u32>,
    pub action: StrictChunkSimulationAction,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrictChunkSimulationAction {
    ChunkPublished,
    RelayCrashedBeforeManifest,
    ChunkReplayed,
    ManifestPublished,
    RelayCrashedBeforeSourceAck,
    ManifestReplayed,
    SourceAcked,
    ManifestArrivedBeforeChunks,
    ChunkWithheld,
    TargetWaitedForMissingChunk,
    MissingChunkReplayed,
    TargetWaitedForManifest,
    TargetStagedAfterManifest,
    TargetCrashedBeforeCommit,
    TargetDiscardedUncommittedStage,
    TargetAppliedAfterManifest,
}

pub fn run_strict_chunk_simulation(
    config: StrictChunkSimulationConfig,
) -> StrictChunkSimulationReport {
    let transaction = generated_transactions(config.seed, 1)
        .into_iter()
        .next()
        .expect("one transaction");
    let mut state = StrictChunkSimState::new(config, transaction);
    state.run();
    state.report()
}

pub fn run_default_strict_chunk_suite(seed: u64) -> Vec<StrictChunkSimulationReport> {
    StrictChunkFailurePoint::ALL
        .into_iter()
        .enumerate()
        .map(|(index, failure_point)| {
            run_strict_chunk_simulation(StrictChunkSimulationConfig::new(
                seed.wrapping_add(200 + index as u64),
                failure_point,
            ))
        })
        .collect()
}
