use std::collections::BTreeSet;

use trellara_protocol::TransactionEnvelope;

use crate::epoch_accumulator::{
    advance_partition_lsn, advance_source_lsn, checked_epoch_count_add,
};
use crate::epoch_summary_builder::EpochSummaryBuilder;
use crate::LakeError;

impl EpochSummaryBuilder {
    pub(crate) fn accumulate_epoch_counts(
        &mut self,
        envelope: &TransactionEnvelope,
    ) -> Result<(), LakeError> {
        self.transaction_count =
            checked_epoch_count_add(self.transaction_count, 1, "transaction_count")?;
        self.change_count =
            checked_epoch_count_add(self.change_count, envelope.changes.len(), "change_count")?;
        self.checksum_rollup ^= envelope.checksum;
        Ok(())
    }

    pub(crate) fn accumulate_source(
        &mut self,
        envelope: &TransactionEnvelope,
    ) -> Result<(), LakeError> {
        let source = self.sources.entry(envelope.source_id.clone()).or_default();
        source.transaction_count =
            checked_epoch_count_add(source.transaction_count, 1, "source.transaction_count")?;
        source.change_count = checked_epoch_count_add(
            source.change_count,
            envelope.changes.len(),
            "source.change_count",
        )?;
        source.checksum_rollup ^= envelope.checksum;
        advance_source_lsn(source, &envelope.commit_lsn)
    }

    pub(crate) fn accumulate_tables(
        &mut self,
        envelope: &TransactionEnvelope,
    ) -> Result<(), LakeError> {
        let transaction_table_key = format!(
            "{}:{}:{}",
            envelope.source_id, envelope.transaction_id, envelope.commit_lsn
        );
        let mut relations_seen_in_transaction = BTreeSet::new();

        for change in &envelope.changes {
            if let Some(relation) = &change.relation {
                let relation = relation.display_name();
                let table = self.tables.entry(relation.clone()).or_default();
                table.change_count =
                    checked_epoch_count_add(table.change_count, 1, "table.change_count")?;
                table.checksum_rollup ^= envelope.checksum;
                relations_seen_in_transaction.insert(relation);
            }
        }
        for relation in relations_seen_in_transaction {
            if let Some(table) = self.tables.get_mut(&relation) {
                table.transactions.insert(transaction_table_key.clone());
            }
        }
        Ok(())
    }

    pub(crate) fn accumulate_partitions(
        &mut self,
        envelope: &TransactionEnvelope,
    ) -> Result<(), LakeError> {
        let Some(manifest) = &envelope.manifest else {
            return Ok(());
        };

        for manifest_partition in &manifest.partitions {
            let partition = self
                .partitions
                .entry((envelope.source_id.clone(), manifest_partition.id))
                .or_default();
            partition.transaction_count = checked_epoch_count_add(
                partition.transaction_count,
                1,
                "partition.transaction_count",
            )?;
            partition.event_count = checked_epoch_count_add(
                partition.event_count,
                manifest_partition.event_count as usize,
                "partition.event_count",
            )?;
            partition.checksum_rollup ^= manifest_partition.checksum;
            advance_partition_lsn(partition, &envelope.commit_lsn)?;
        }
        Ok(())
    }
}
