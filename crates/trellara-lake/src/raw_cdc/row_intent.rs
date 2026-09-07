use trellara_protocol::{ChangeRecord, Operation, RowImage, TransactionEnvelope};

use crate::commit_row::{lake_value_string, primary_key_value, project_row};
use crate::raw_cdc::ddl_metadata::ddl_metadata;
use crate::raw_cdc::row_manifest::{
    manifest_boundary_mode, manifest_id, manifest_participating_partition_count,
};
use crate::raw_cdc_types::LakeRawCdcRowIntent;
use crate::{LakeColumnValue, LakeError, LakeTableConfig};

pub(super) fn row_intent(
    envelope: &TransactionEnvelope,
    epoch_id: &str,
    source_bucket: u32,
    table: &LakeTableConfig,
    change: &ChangeRecord,
) -> Result<LakeRawCdcRowIntent, LakeError> {
    let relation = change.relation.as_ref().ok_or(LakeError::MissingRelation {
        total_order: change.total_order,
    })?;
    let image = change.after.as_ref().or(change.before.as_ref());
    let ddl_metadata = ddl_metadata(envelope, relation);

    Ok(LakeRawCdcRowIntent {
        source_id: envelope.source_id.clone(),
        source_bucket,
        database_id: envelope.database_id.clone(),
        dataset_id: envelope.dataset_id.clone(),
        relation: relation.display_name(),
        transaction_id: envelope.transaction_id.clone(),
        begin_lsn: envelope.begin_lsn.clone(),
        commit_lsn: envelope.commit_lsn.clone(),
        commit_timestamp_ms: envelope.commit_timestamp_ms,
        total_order: change.total_order,
        operation: operation_label(change)?,
        record_key: image
            .map(|row| primary_key_value(row, table, change.total_order))
            .transpose()?,
        idempotency_key: change.idempotency_key.clone(),
        schema_fingerprint: table.source_schema_fingerprint,
        schema_version: ddl_metadata.schema_version,
        ddl_barrier_id: ddl_metadata.ddl_barrier_id,
        ddl_release_gate: ddl_metadata.ddl_release_gate,
        ddl_schema_fingerprint_before: ddl_metadata.schema_fingerprint_before,
        ddl_schema_fingerprint_after: ddl_metadata.schema_fingerprint_after,
        envelope_checksum: envelope.checksum,
        manifest_id: envelope.manifest.as_ref().map(manifest_id),
        manifest_boundary_mode: manifest_boundary_mode(envelope.manifest.as_ref())?,
        manifest_global_event_count: envelope
            .manifest
            .as_ref()
            .map(|manifest| manifest.global_event_count),
        manifest_participating_partition_count: manifest_participating_partition_count(
            envelope.manifest.as_ref(),
        )?,
        partition_key: partition_key_value(image, table)?,
        epoch_id: epoch_id.to_string(),
        ingested_at: "planned_after_raw_cdc_files_durable".to_string(),
        payload_before: project_optional_row(change.before.as_ref(), table)?,
        payload_after: project_optional_row(change.after.as_ref(), table)?,
    })
}

fn partition_key_value(
    row: Option<&RowImage>,
    table: &LakeTableConfig,
) -> Result<Option<String>, LakeError> {
    let Some(column_name) = table.partition_key_column.as_ref() else {
        return Ok(None);
    };
    let Some(row) = row else {
        return Ok(None);
    };
    row.columns
        .iter()
        .find(|column| column.name == *column_name)
        .map(lake_value_string)
        .transpose()
}

fn project_optional_row(
    row: Option<&RowImage>,
    table: &LakeTableConfig,
) -> Result<Vec<LakeColumnValue>, LakeError> {
    row.map(|row| project_row(row, table))
        .transpose()
        .map(Option::unwrap_or_default)
}

fn operation_label(change: &ChangeRecord) -> Result<String, LakeError> {
    let operation =
        Operation::try_from(change.operation).map_err(|_| LakeError::UnsupportedOperation {
            total_order: change.total_order,
            operation: change.operation,
        })?;
    Ok(match operation {
        Operation::Insert => "insert",
        Operation::Update => "update",
        Operation::Delete => "delete",
        Operation::Truncate => "truncate",
        Operation::Unspecified => {
            return Err(LakeError::UnsupportedOperation {
                total_order: change.total_order,
                operation: change.operation,
            });
        }
    }
    .to_string())
}
