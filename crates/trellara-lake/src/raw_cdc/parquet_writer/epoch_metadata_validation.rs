use std::collections::BTreeMap;

use crate::LakeError;

pub(super) fn validate_snapshot_evidence(
    iceberg_snapshot_id: &str,
    raw_table_snapshot_ids: &BTreeMap<String, i64>,
) -> Result<(), LakeError> {
    if iceberg_snapshot_id.trim().is_empty() || iceberg_snapshot_id != iceberg_snapshot_id.trim() {
        return invalid_metadata_field(
            "epoch.iceberg_snapshot_id",
            "must be non-empty and trimmed before _trellara_epochs is written",
        );
    }
    if raw_table_snapshot_ids.is_empty() {
        return invalid_metadata_field(
            "epoch.raw_table_snapshot_ids",
            "must include every raw changelog table snapshot before _trellara_epochs is written",
        );
    }
    for (target, snapshot_id) in raw_table_snapshot_ids {
        if target.trim().is_empty() || target != target.trim() {
            return invalid_metadata_field(
                "epoch.raw_table_snapshot_ids.target",
                "must be non-empty and trimmed",
            );
        }
        if *snapshot_id <= 0 {
            return invalid_metadata_field(
                "epoch.raw_table_snapshot_ids.snapshot_id",
                "must be greater than zero",
            );
        }
    }
    Ok(())
}

fn invalid_metadata_field<T>(field: &'static str, reason: &str) -> Result<T, LakeError> {
    Err(LakeError::InvalidRawCdcEpochMetadataField {
        field,
        reason: reason.to_string(),
    })
}
