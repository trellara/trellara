use std::collections::{BTreeMap, BTreeSet};

use trellara_protocol::TransactionEnvelope;

use crate::epoch::{LakeEpochConfig, LakeEpochSourceState};
use crate::epoch_accumulator::{PartitionAccumulator, SourceAccumulator, TableAccumulator};
use crate::epoch_replay_ledger::EpochReplayLedger;
use crate::{validate_epoch_manifest, LakeError};

pub(crate) struct EpochSummaryBuilder {
    pub(crate) sources: BTreeMap<String, SourceAccumulator>,
    pub(crate) tables: BTreeMap<String, TableAccumulator>,
    pub(crate) partitions: BTreeMap<(String, u32), PartitionAccumulator>,
    replay_ledger: EpochReplayLedger,
    pub(crate) transaction_count: usize,
    pub(crate) change_count: usize,
    pub(crate) checksum_rollup: u64,
    dataset_id: String,
    required_sources: BTreeSet<String>,
}

impl EpochSummaryBuilder {
    pub(crate) fn new(config: &LakeEpochConfig) -> Self {
        Self {
            sources: config
                .required_sources
                .iter()
                .map(|source_id| (source_id.clone(), SourceAccumulator::default()))
                .collect(),
            tables: BTreeMap::new(),
            partitions: BTreeMap::new(),
            replay_ledger: EpochReplayLedger::default(),
            transaction_count: 0,
            change_count: 0,
            checksum_rollup: 0,
            dataset_id: config.dataset_id.clone(),
            required_sources: config.required_sources.clone(),
        }
    }

    pub(crate) fn ingest_all(
        &mut self,
        envelopes: &[TransactionEnvelope],
    ) -> Result<(), LakeError> {
        for envelope in envelopes {
            self.ingest(envelope)?;
        }
        Ok(())
    }

    fn ingest(&mut self, envelope: &TransactionEnvelope) -> Result<(), LakeError> {
        envelope.verify_checksum()?;
        envelope.validate()?;
        validate_epoch_manifest(envelope)?;

        if envelope.dataset_id != self.dataset_id {
            return Ok(());
        }
        if !self.source_in_scope(&envelope.source_id) {
            return Ok(());
        }

        if let Err(error) = self.replay_ledger.validate_idempotency(envelope) {
            self.quarantine_source_for_conflicting_duplicate(envelope, &error);
            return Err(error);
        }
        if self.replay_ledger.is_replayed_transaction(envelope) {
            return Ok(());
        }

        self.accumulate_epoch_counts(envelope)?;
        self.accumulate_source(envelope)?;
        self.accumulate_tables(envelope)?;
        self.accumulate_partitions(envelope)
    }

    fn source_in_scope(&self, source_id: &str) -> bool {
        self.required_sources.is_empty() || self.required_sources.contains(source_id)
    }

    fn quarantine_source_for_conflicting_duplicate(
        &mut self,
        envelope: &TransactionEnvelope,
        error: &LakeError,
    ) {
        let LakeError::ConflictingDuplicate { idempotency_key } = error else {
            return;
        };
        let source = self.sources.entry(envelope.source_id.clone()).or_default();
        source.state = LakeEpochSourceState::Quarantined;
        source.gap_reason = Some(format!(
            "conflicting duplicate idempotency key {idempotency_key}"
        ));
    }
}
