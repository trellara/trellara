use crate::raw_cdc::verification_id;
use crate::{
    consumer_gate_completeness::validate_epoch_completeness_state, LakeError, LakeRawCdcEpochRow,
    LakeRawCdcEpochVerificationRow,
};

pub(crate) fn validate_epoch_metadata_identity(
    epoch: &LakeRawCdcEpochRow,
    verification: &LakeRawCdcEpochVerificationRow,
) -> Result<(), LakeError> {
    validate_clean_epoch_metadata_field("epoch.epoch_id", &epoch.epoch_id)?;
    validate_clean_epoch_metadata_field("epoch.dataset_id", &epoch.dataset_id)?;
    validate_clean_epoch_metadata_field("epoch.policy", &epoch.policy)?;
    validate_clean_epoch_metadata_field("epoch.opened_at", &epoch.opened_at)?;
    validate_clean_epoch_metadata_field("epoch.sealed_at", &epoch.sealed_at)?;
    validate_clean_epoch_metadata_field("epoch.manifest_digest", &epoch.manifest_digest)?;
    validate_manifest_digest_shape(&epoch.manifest_digest)?;
    if let Some(snapshot_id) = &epoch.iceberg_snapshot_id {
        validate_clean_epoch_metadata_field("epoch.iceberg_snapshot_id", snapshot_id)?;
    }
    validate_clean_epoch_metadata_field("verification.epoch_id", &verification.epoch_id)?;
    validate_clean_epoch_metadata_field(
        "verification.verification_id",
        &verification.verification_id,
    )?;
    validate_clean_epoch_metadata_field("verification.completed_at", &verification.completed_at)?;
    validate_verification_id(epoch, verification)
}

pub(crate) fn validate_epoch_consistency(epoch: &LakeRawCdcEpochRow) -> Result<(), LakeError> {
    validate_source_count_rollup(epoch)?;
    validate_epoch_completeness_state(epoch)
}

fn validate_clean_epoch_metadata_field(field: &'static str, value: &str) -> Result<(), LakeError> {
    if value.trim().is_empty() {
        return Err(LakeError::InvalidRawCdcEpochMetadataField {
            field,
            reason: "must not be empty".to_string(),
        });
    }
    if value != value.trim() {
        return Err(LakeError::InvalidRawCdcEpochMetadataField {
            field,
            reason: "must not contain surrounding whitespace".to_string(),
        });
    }
    Ok(())
}

fn validate_verification_id(
    epoch: &LakeRawCdcEpochRow,
    verification: &LakeRawCdcEpochVerificationRow,
) -> Result<(), LakeError> {
    let expected = verification_id(&epoch.dataset_id, &epoch.epoch_id, epoch.checksum_rollup);
    if verification.verification_id == expected {
        return Ok(());
    }
    Err(LakeError::InvalidRawCdcEpochMetadataField {
        field: "verification.verification_id",
        reason: format!("must equal {expected}"),
    })
}

fn validate_manifest_digest_shape(manifest_digest: &str) -> Result<(), LakeError> {
    if manifest_digest.len() == 64 && manifest_digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Ok(());
    }
    Err(LakeError::InvalidRawCdcEpochMetadataField {
        field: "epoch.manifest_digest",
        reason: "must be a 64-character hex digest".to_string(),
    })
}

pub(crate) fn validate_verification_counts(
    epoch: &LakeRawCdcEpochRow,
    verification: &LakeRawCdcEpochVerificationRow,
) -> Result<(), LakeError> {
    if verification.input_transaction_count != epoch.transaction_count
        || verification.lake_transaction_count != epoch.transaction_count
    {
        return Err(LakeError::EpochVerificationCountMismatch {
            epoch_id: epoch.epoch_id.clone(),
            field: "transaction_count",
            epoch_count: epoch.transaction_count,
            input_count: verification.input_transaction_count,
            lake_count: verification.lake_transaction_count,
        });
    }
    if verification.input_change_count != epoch.change_count
        || verification.lake_change_count != epoch.change_count
    {
        return Err(LakeError::EpochVerificationCountMismatch {
            epoch_id: epoch.epoch_id.clone(),
            field: "change_count",
            epoch_count: epoch.change_count,
            input_count: verification.input_change_count,
            lake_count: verification.lake_change_count,
        });
    }
    Ok(())
}

fn validate_source_count_rollup(epoch: &LakeRawCdcEpochRow) -> Result<(), LakeError> {
    let observed_sources = epoch
        .complete_source_count
        .saturating_add(epoch.missing_source_count)
        .saturating_add(epoch.quarantined_source_count);
    if observed_sources == epoch.required_source_count {
        Ok(())
    } else {
        Err(LakeError::EpochSourceCountMismatch {
            epoch_id: epoch.epoch_id.clone(),
            required_source_count: epoch.required_source_count,
            complete_source_count: epoch.complete_source_count,
            missing_source_count: epoch.missing_source_count,
            quarantined_source_count: epoch.quarantined_source_count,
        })
    }
}
