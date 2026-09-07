use trellara_protocol::{ChangeRecord, RowImage, TransactionEnvelope};

use super::ddl_metadata_validation::validate_ddl_schema_evidence;
use super::dedup::{RawCdcDedupLedger, RawCdcDedupOutcome};
use crate::commit_materialization_operation::validate_change_operation;
use crate::commit_row::value_kind;
use crate::raw_cdc_accumulator::checked_raw_cdc_count_add;
use crate::{validate_epoch_manifest, LakeError};

pub(super) enum RawCdcEnvelopeAdmission {
    Accumulate { transaction_id: String },
    Duplicate,
    SkippedDataset,
}

#[derive(Default)]
pub(super) struct RawCdcPlannerCounters {
    pub(super) transaction_count: usize,
    pub(super) change_count: usize,
    pub(super) duplicate_transaction_count: usize,
    pub(super) skipped_dataset_transaction_count: usize,
    pub(super) checksum_rollup: u64,
}

pub(super) fn admit_envelope(
    envelope: &TransactionEnvelope,
    dataset_id: &str,
    dedup: &mut RawCdcDedupLedger,
    counters: &mut RawCdcPlannerCounters,
) -> Result<RawCdcEnvelopeAdmission, LakeError> {
    envelope.verify_checksum()?;

    if envelope.dataset_id != dataset_id {
        counters.skipped_dataset_transaction_count = checked_raw_cdc_count_add(
            counters.skipped_dataset_transaction_count,
            1,
            "skipped_dataset_transaction_count",
        )?;
        return Ok(RawCdcEnvelopeAdmission::SkippedDataset);
    }

    envelope.validate()?;
    validate_ddl_schema_evidence(envelope)?;
    validate_epoch_manifest(envelope)?;
    validate_change_operations(envelope)?;

    match dedup.record(envelope)? {
        RawCdcDedupOutcome::New { transaction_id } => {
            counters.transaction_count =
                checked_raw_cdc_count_add(counters.transaction_count, 1, "transaction_count")?;
            counters.change_count = checked_raw_cdc_count_add(
                counters.change_count,
                envelope.changes.len(),
                "change_count",
            )?;
            counters.checksum_rollup ^= envelope.checksum;
            Ok(RawCdcEnvelopeAdmission::Accumulate { transaction_id })
        }
        RawCdcDedupOutcome::Duplicate => {
            counters.duplicate_transaction_count = checked_raw_cdc_count_add(
                counters.duplicate_transaction_count,
                1,
                "duplicate_transaction_count",
            )?;
            Ok(RawCdcEnvelopeAdmission::Duplicate)
        }
    }
}

fn validate_change_operations(envelope: &TransactionEnvelope) -> Result<(), LakeError> {
    for change in &envelope.changes {
        validate_change_operation(change)?;
        validate_change_row_values(change)?;
    }
    Ok(())
}

fn validate_change_row_values(change: &ChangeRecord) -> Result<(), LakeError> {
    if let Some(before) = &change.before {
        validate_row_values(before)?;
    }
    if let Some(after) = &change.after {
        validate_row_values(after)?;
    }
    Ok(())
}

fn validate_row_values(row: &RowImage) -> Result<(), LakeError> {
    for column in &row.columns {
        value_kind(column)?;
    }
    Ok(())
}
