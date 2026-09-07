use std::collections::BTreeMap;
use std::sync::Arc;

use arrow_array::{ArrayRef, Int64Array, RecordBatch, StringArray};

use super::epoch_metadata_schema::raw_cdc_epoch_metadata_parquet_arrow_schema;
use crate::{LakeCompletenessState, LakeError, LakeRawCdcEpochWritePlan};

pub(super) fn record_batch(
    write_plan: &LakeRawCdcEpochWritePlan,
    iceberg_snapshot_id: &str,
    raw_table_snapshot_ids: &BTreeMap<String, i64>,
) -> Result<RecordBatch, LakeError> {
    let row = &write_plan.epoch_metadata.epoch_row;
    let raw_table_snapshot_ids_json =
        serde_json::to_string(raw_table_snapshot_ids).map_err(|error| {
            LakeError::RawCdcParquet {
                operation: "raw_table_snapshot_ids_json",
                message: error.to_string(),
            }
        })?;

    RecordBatch::try_new(
        raw_cdc_epoch_metadata_parquet_arrow_schema(),
        vec![
            one_string(&row.epoch_id),
            one_string(&row.dataset_id),
            one_string(completeness_state_label(row.state)),
            one_string(&row.policy),
            one_string(&row.opened_at),
            one_string(&row.sealed_at),
            one_i64("required_source_count", row.required_source_count)?,
            one_i64("complete_source_count", row.complete_source_count)?,
            one_i64("missing_source_count", row.missing_source_count)?,
            one_i64("quarantined_source_count", row.quarantined_source_count)?,
            one_i64("transaction_count", row.transaction_count)?,
            one_i64("change_count", row.change_count)?,
            one_checksum(row.checksum_rollup),
            one_string(&row.manifest_digest),
            one_string(iceberg_snapshot_id),
            one_string(&raw_table_snapshot_ids_json),
        ],
    )
    .map_err(|error| LakeError::RawCdcParquet {
        operation: "build_epoch_metadata_record_batch",
        message: error.to_string(),
    })
}

fn one_string(value: &str) -> ArrayRef {
    Arc::new(StringArray::from_iter_values([value])) as ArrayRef
}

fn one_i64(field: &'static str, value: usize) -> Result<ArrayRef, LakeError> {
    let value =
        i64::try_from(value).map_err(|_| LakeError::RawCdcParquetCountOverflow { field })?;
    Ok(Arc::new(Int64Array::from_iter_values([value])) as ArrayRef)
}

fn one_checksum(value: u64) -> ArrayRef {
    Arc::new(Int64Array::from_iter_values([i64::from_ne_bytes(
        value.to_ne_bytes(),
    )])) as ArrayRef
}

fn completeness_state_label(state: LakeCompletenessState) -> &'static str {
    match state {
        LakeCompletenessState::Open => "open",
        LakeCompletenessState::Sealing => "sealing",
        LakeCompletenessState::Complete => "complete",
        LakeCompletenessState::CompleteWithGaps => "complete_with_gaps",
        LakeCompletenessState::Quarantined => "quarantined",
        LakeCompletenessState::Reseeding => "reseeding",
        LakeCompletenessState::FailedRecoverable => "failed_recoverable",
    }
}
