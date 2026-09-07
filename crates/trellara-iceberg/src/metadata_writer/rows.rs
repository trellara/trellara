use std::collections::BTreeMap;

use trellara_lake::LakeRawCdcEpochWritePlan;

use super::values::{
    checksum, completeness_state, count, integer, optional_string, source_state, string,
    verification_status, MetadataValue,
};
use crate::{IcebergIntegrationError, IcebergMetadataTableKind, Result};

pub(super) fn metadata_rows(
    write_plan: &LakeRawCdcEpochWritePlan,
    epoch_snapshot_reference: &str,
    raw_snapshot_ids: &BTreeMap<String, i64>,
) -> Result<Vec<(IcebergMetadataTableKind, Vec<Vec<MetadataValue>>)>> {
    let metadata = &write_plan.epoch_metadata;
    let raw_snapshot_ids_json = serde_json::to_string(raw_snapshot_ids).map_err(|error| {
        IcebergIntegrationError::MetadataEncoding {
            table: metadata.epochs_table.clone(),
            message: error.to_string(),
        }
    })?;
    let sources = metadata
        .source_rows
        .iter()
        .map(|row| {
            Ok(vec![
                string(&row.epoch_id),
                string(&row.source_id),
                string(source_state(row.state)),
                string(&row.start_lsn),
                string(&row.end_lsn),
                count("source.transaction_count", row.transaction_count)?,
                count("source.change_count", row.change_count)?,
                checksum(row.checksum_rollup),
                optional_string(row.lag_reason.as_deref()),
            ])
        })
        .collect::<Result<Vec<_>>>()?;
    let tables = metadata
        .table_rows
        .iter()
        .map(|row| {
            Ok(vec![
                string(&row.epoch_id),
                string(&row.relation),
                count("table.transaction_count", row.transaction_count)?,
                count("table.change_count", row.change_count)?,
                checksum(row.checksum_rollup),
            ])
        })
        .collect::<Result<Vec<_>>>()?;
    let partitions = metadata
        .partition_rows
        .iter()
        .map(|row| {
            Ok(vec![
                string(&row.epoch_id),
                string(&row.source_id),
                integer("partition.partition_id", row.partition_id)?,
                string(&row.first_commit_lsn),
                string(&row.last_commit_lsn),
                count("partition.transaction_count", row.transaction_count)?,
                count("partition.event_count", row.event_count)?,
                checksum(row.checksum_rollup),
            ])
        })
        .collect::<Result<Vec<_>>>()?;
    let quarantine = metadata
        .quarantine_rows
        .iter()
        .map(|row| {
            vec![
                string(&row.epoch_id),
                optional_string(row.source_id.as_deref()),
                optional_string(row.transaction_id.as_deref()),
                optional_string(row.commit_lsn.as_deref()),
                string(&row.reason),
                optional_string(row.details.as_deref()),
                optional_string(row.recovery_command.as_deref()),
            ]
        })
        .collect::<Vec<_>>();
    let verification = &metadata.verification_row;
    let verification_rows = vec![vec![
        string(&verification.epoch_id),
        string(&verification.verification_id),
        count(
            "verification.input_transaction_count",
            verification.input_transaction_count,
        )?,
        count(
            "verification.input_change_count",
            verification.input_change_count,
        )?,
        count(
            "verification.lake_transaction_count",
            verification.lake_transaction_count,
        )?,
        count(
            "verification.lake_change_count",
            verification.lake_change_count,
        )?,
        string(verification_status(verification.checksum_status)),
        string(&verification.completed_at),
    ]];
    let epoch = &metadata.epoch_row;
    let completeness = vec![vec![
        string(&epoch.epoch_id),
        string(&epoch.dataset_id),
        string(completeness_state(epoch.state)),
        string(&epoch.policy),
        string(&epoch.opened_at),
        string(&epoch.sealed_at),
        count("epoch.required_source_count", epoch.required_source_count)?,
        count("epoch.complete_source_count", epoch.complete_source_count)?,
        count("epoch.missing_source_count", epoch.missing_source_count)?,
        count(
            "epoch.quarantined_source_count",
            epoch.quarantined_source_count,
        )?,
        count("epoch.transaction_count", epoch.transaction_count)?,
        count("epoch.change_count", epoch.change_count)?,
        checksum(epoch.checksum_rollup),
        string(&epoch.manifest_digest),
        string(epoch_snapshot_reference),
        string(&raw_snapshot_ids_json),
    ]];
    Ok(vec![
        (IcebergMetadataTableKind::Source, sources),
        (IcebergMetadataTableKind::Table, tables),
        (IcebergMetadataTableKind::Partition, partitions),
        (IcebergMetadataTableKind::Quarantine, quarantine),
        (IcebergMetadataTableKind::Verification, verification_rows),
        (IcebergMetadataTableKind::Completeness, completeness),
    ])
}
