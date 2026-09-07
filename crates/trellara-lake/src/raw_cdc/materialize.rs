use crate::raw_cdc::commit_steps::commit_steps;
use crate::raw_cdc::committer::committer_topology;
use crate::raw_cdc::metadata::{
    build_epoch_metadata, RawCdcMetadataInput, RAW_CDC_VISIBILITY_RULE,
};
use crate::raw_cdc::planner::RawCdcEpochPlanner;
use crate::raw_cdc::recovery::recovery_scenarios;
use crate::raw_cdc_naming::raw_cdc_object_key_hint;
use crate::raw_cdc_types::{
    LakeRawCdcDataFilePlan, LakeRawCdcDuplicateReplayEvidence, LakeRawCdcEpochPartitionRow,
    LakeRawCdcEpochSourceRow, LakeRawCdcEpochTableRow, LakeRawCdcEpochWritePlan,
};
use crate::LakeEpochSourceState;

impl RawCdcEpochPlanner<'_> {
    pub(super) fn into_write_plan(self) -> Result<LakeRawCdcEpochWritePlan, crate::LakeError> {
        let data_files = self.data_files();
        let source_rows = self.source_rows();
        let table_rows = self.table_rows();
        let partition_rows = self.partition_rows();

        Ok(LakeRawCdcEpochWritePlan {
            dataset_id: self.writer_config.dataset_id.clone(),
            epoch_id: self.writer_config.epoch_id.clone(),
            transaction_count: self.counters.transaction_count,
            change_count: self.counters.change_count,
            duplicate_transaction_count: self.counters.duplicate_transaction_count,
            skipped_dataset_transaction_count: self.counters.skipped_dataset_transaction_count,
            data_file_count: data_files.len(),
            checksum_rollup: self.counters.checksum_rollup,
            visibility_rule: RAW_CDC_VISIBILITY_RULE.to_string(),
            duplicate_replay_evidence: duplicate_replay_evidence(
                self.counters.duplicate_transaction_count,
                self.counters.transaction_count,
                self.rows.len(),
            ),
            committer_topology: committer_topology(
                &data_files,
                self.writer_config.source_bucket_count,
            ),
            commit_steps: commit_steps(&data_files, &self.writer_config.epoch_id),
            recovery_scenarios: recovery_scenarios(&self.writer_config.epoch_id),
            data_files,
            row_intents: self.rows,
            epoch_metadata: build_epoch_metadata(RawCdcMetadataInput {
                dataset_id: self.writer_config.dataset_id.clone(),
                epoch_id: self.writer_config.epoch_id.clone(),
                transaction_count: self.counters.transaction_count,
                change_count: self.counters.change_count,
                checksum_rollup: self.counters.checksum_rollup,
                required_sources: self.writer_config.required_sources.clone(),
                straggler_policy: self.writer_config.straggler_policy.clone(),
                source_rows,
                table_rows,
                partition_rows,
            })?,
        })
    }

    fn data_files(&self) -> Vec<LakeRawCdcDataFilePlan> {
        self.files
            .values()
            .map(|file| LakeRawCdcDataFilePlan {
                object_key_hint: raw_cdc_object_key_hint(
                    &self.writer_config.epoch_id,
                    &file.table_name,
                    file.source_bucket,
                ),
                table_name: file.table_name.clone(),
                relation: file.relation.clone(),
                source_bucket: file.source_bucket,
                source_ids: file.source_ids.iter().cloned().collect(),
                transaction_count: file.transactions.len(),
                change_count: file.change_count,
                min_commit_lsn: file.min_commit_lsn.clone().unwrap_or_default(),
                max_commit_lsn: file.max_commit_lsn.clone().unwrap_or_default(),
                checksum_rollup: file.checksum_rollup,
                idempotency_key_count: file.idempotency_keys.len(),
            })
            .collect()
    }

    fn source_rows(&self) -> Vec<LakeRawCdcEpochSourceRow> {
        self.sources
            .iter()
            .map(|(source_id, source)| LakeRawCdcEpochSourceRow {
                epoch_id: self.writer_config.epoch_id.clone(),
                source_id: source_id.clone(),
                start_lsn: source.start_lsn.clone().unwrap_or_default(),
                end_lsn: source.end_lsn.clone().unwrap_or_default(),
                transaction_count: source.transactions.len(),
                change_count: source.change_count,
                checksum_rollup: source.checksum_rollup,
                state: LakeEpochSourceState::Complete,
                lag_reason: None,
            })
            .collect()
    }

    fn table_rows(&self) -> Vec<LakeRawCdcEpochTableRow> {
        self.tables
            .iter()
            .map(|(relation, table)| LakeRawCdcEpochTableRow {
                epoch_id: self.writer_config.epoch_id.clone(),
                relation: relation.clone(),
                transaction_count: table.transactions.len(),
                change_count: table.change_count,
                checksum_rollup: table.checksum_rollup,
            })
            .collect()
    }

    fn partition_rows(&self) -> Vec<LakeRawCdcEpochPartitionRow> {
        self.partitions
            .iter()
            .map(
                |((source_id, partition_id), partition)| LakeRawCdcEpochPartitionRow {
                    epoch_id: self.writer_config.epoch_id.clone(),
                    source_id: source_id.clone(),
                    partition_id: *partition_id,
                    first_commit_lsn: partition.first_commit_lsn.clone().unwrap_or_default(),
                    last_commit_lsn: partition.last_commit_lsn.clone().unwrap_or_default(),
                    transaction_count: partition.transactions.len(),
                    event_count: partition.event_count,
                    checksum_rollup: partition.checksum_rollup,
                },
            )
            .collect()
    }
}

fn duplicate_replay_evidence(
    duplicate_transaction_count: usize,
    unique_transaction_count: usize,
    row_intent_count: usize,
) -> LakeRawCdcDuplicateReplayEvidence {
    LakeRawCdcDuplicateReplayEvidence {
        contract: "duplicate replay is skipped by transaction identity and conflicting idempotency evidence fails closed".to_string(),
        duplicate_transaction_count,
        unique_transaction_count,
        row_intent_count,
        idempotency_key_count: row_intent_count,
        replay_safe: true,
    }
}
