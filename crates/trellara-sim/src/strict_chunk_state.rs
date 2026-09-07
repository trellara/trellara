use crate::failure_injection::FailureInjection;
use crate::strict_chunk::{StrictChunkSimulationConfig, StrictChunkSimulationReport};
use crate::strict_chunk_report::{build_strict_chunk_report, StrictChunkReportInput};
use crate::strict_chunk_steps::StrictChunkSimulationSteps;
use crate::strict_chunk_tracker::StrictChunkTracker;
use crate::transaction::Transaction;

pub(crate) struct StrictChunkSimState {
    pub(crate) config: StrictChunkSimulationConfig,
    pub(crate) transaction: Transaction,
    pub(crate) chunks: StrictChunkTracker,
    pub(crate) manifest_published: bool,
    pub(crate) source_acknowledged_lsn: Option<u64>,
    pub(crate) target_applied_lsn: Option<u64>,
    pub(crate) applied_transactions: usize,
    pub(crate) failure: FailureInjection,
    pub(crate) steps: StrictChunkSimulationSteps,
}

impl StrictChunkSimState {
    pub(crate) fn new(config: StrictChunkSimulationConfig, transaction: Transaction) -> Self {
        Self {
            config,
            transaction,
            chunks: StrictChunkTracker::default(),
            manifest_published: false,
            source_acknowledged_lsn: None,
            target_applied_lsn: None,
            applied_transactions: 0,
            failure: FailureInjection::default(),
            steps: StrictChunkSimulationSteps::default(),
        }
    }

    pub(crate) fn report(self) -> StrictChunkSimulationReport {
        build_strict_chunk_report(StrictChunkReportInput {
            config: self.config,
            transaction_id: self.transaction.id,
            commit_lsn: self.transaction.lsn,
            chunks_published: self.chunks.published_count(),
            duplicate_chunks: self.chunks.duplicate_count(),
            manifest_published: self.manifest_published,
            source_acknowledged_lsn: self.source_acknowledged_lsn,
            target_applied_lsn: self.target_applied_lsn,
            applied_transactions: self.applied_transactions,
            injected_failure: self.failure.into_description(),
            steps: self.steps.into_steps(),
        })
    }
}
