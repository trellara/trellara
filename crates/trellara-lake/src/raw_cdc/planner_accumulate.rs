use std::collections::BTreeMap;

use trellara_protocol::TransactionEnvelope;

use super::file_change::accumulate_file_change;
use super::planner::RawCdcEpochPlanner;
use super::row_intent::row_intent;
use crate::raw_cdc_accumulator::{advance_raw_cdc_lsn, checked_raw_cdc_count_add};
use crate::raw_cdc_naming::source_bucket;
use crate::LakeError;

impl RawCdcEpochPlanner<'_> {
    pub(super) fn accumulate_source(
        &mut self,
        envelope: &TransactionEnvelope,
        transaction_id: &str,
    ) -> Result<(), LakeError> {
        if transaction_id.is_empty() {
            return Ok(());
        }

        let source = self.sources.entry(envelope.source_id.clone()).or_default();
        source.transactions.insert(transaction_id.to_string());
        source.change_count = checked_raw_cdc_count_add(
            source.change_count,
            envelope.changes.len(),
            "source.change_count",
        )?;
        source.checksum_rollup ^= envelope.checksum;
        advance_raw_cdc_lsn(
            &mut source.start_lsn,
            &mut source.end_lsn,
            &envelope.commit_lsn,
        )
    }

    pub(super) fn accumulate_files_and_tables(
        &mut self,
        envelope: &TransactionEnvelope,
        transaction_id: &str,
    ) -> Result<(), LakeError> {
        if transaction_id.is_empty() {
            return Ok(());
        }

        let source_bucket =
            source_bucket(&envelope.source_id, self.writer_config.source_bucket_count);
        let relation_change_counts =
            self.accumulate_rows_and_files(envelope, transaction_id, source_bucket)?;
        self.accumulate_tables(transaction_id, envelope.checksum, relation_change_counts)?;
        self.accumulate_partitions(envelope, transaction_id)
    }

    fn accumulate_rows_and_files(
        &mut self,
        envelope: &TransactionEnvelope,
        transaction_id: &str,
        source_bucket: u32,
    ) -> Result<BTreeMap<String, usize>, LakeError> {
        let mut relation_change_counts = BTreeMap::<String, usize>::new();
        for change in &envelope.changes {
            let relation = change.relation.as_ref().ok_or(LakeError::MissingRelation {
                total_order: change.total_order,
            })?;
            let table = self.plan_config.table_for(relation)?;
            self.rows.push(row_intent(
                envelope,
                &self.writer_config.epoch_id,
                source_bucket,
                table,
                change,
            )?);
            let relation_name = accumulate_file_change(
                &mut self.files,
                self.plan_config,
                &self.writer_config.dataset_id,
                envelope,
                transaction_id,
                source_bucket,
                change,
            )?;
            let count = relation_change_counts.entry(relation_name).or_default();
            *count = checked_raw_cdc_count_add(*count, 1, "relation.change_count")?;
        }
        Ok(relation_change_counts)
    }

    fn accumulate_tables(
        &mut self,
        transaction_id: &str,
        checksum: u64,
        relation_change_counts: BTreeMap<String, usize>,
    ) -> Result<(), LakeError> {
        for (relation, relation_change_count) in relation_change_counts {
            let table = self.tables.entry(relation).or_default();
            if table.transactions.insert(transaction_id.to_string()) {
                table.checksum_rollup ^= checksum;
            }
            table.change_count = checked_raw_cdc_count_add(
                table.change_count,
                relation_change_count,
                "table.change_count",
            )?;
        }
        Ok(())
    }

    fn accumulate_partitions(
        &mut self,
        envelope: &TransactionEnvelope,
        transaction_id: &str,
    ) -> Result<(), LakeError> {
        let Some(manifest) = &envelope.manifest else {
            return Ok(());
        };

        for manifest_partition in &manifest.partitions {
            let partition = self
                .partitions
                .entry((envelope.source_id.clone(), manifest_partition.id))
                .or_default();
            partition.transactions.insert(transaction_id.to_string());
            partition.event_count = checked_raw_cdc_count_add(
                partition.event_count,
                manifest_partition.event_count as usize,
                "partition.event_count",
            )?;
            partition.checksum_rollup ^= manifest_partition.checksum;
            advance_raw_cdc_lsn(
                &mut partition.first_commit_lsn,
                &mut partition.last_commit_lsn,
                &envelope.commit_lsn,
            )?;
        }
        Ok(())
    }
}
