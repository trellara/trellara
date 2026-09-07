use crate::{
    parse_lsn, validate_transaction_manifest, ProtocolError, TransactionCommitMarker,
    TransactionManifest,
};

impl TransactionCommitMarker {
    pub fn from_manifest(manifest: &TransactionManifest) -> Result<Self, ProtocolError> {
        validate_transaction_manifest(manifest)?;
        Ok(Self {
            transaction_id: manifest.transaction_id.clone(),
            source_commit_lsn: manifest.source_commit_lsn.clone(),
            source_commit_timestamp_ms: manifest.source_commit_timestamp_ms,
            global_event_count: manifest.global_event_count,
            participating_partition_count: participating_partition_count(manifest)?,
            manifest_checksum: manifest.compute_checksum(),
        })
    }

    pub fn matches_manifest(&self, manifest: &TransactionManifest) -> bool {
        if validate_commit_marker(self).is_err() {
            return false;
        }
        if validate_transaction_manifest(manifest).is_err() {
            return false;
        }
        let Ok(participating_partition_count) = participating_partition_count(manifest) else {
            return false;
        };
        self.transaction_id == manifest.transaction_id
            && self.source_commit_lsn == manifest.source_commit_lsn
            && self.source_commit_timestamp_ms == manifest.source_commit_timestamp_ms
            && self.global_event_count == manifest.global_event_count
            && self.participating_partition_count == participating_partition_count
            && self.manifest_checksum == manifest.compute_checksum()
    }
}

pub fn validate_commit_marker(marker: &TransactionCommitMarker) -> Result<(), ProtocolError> {
    require_commit_marker_field("transaction_id", &marker.transaction_id)?;
    require_commit_marker_field("source_commit_lsn", &marker.source_commit_lsn)?;
    let parsed = parse_lsn(&marker.source_commit_lsn)?;
    if parsed == 0 {
        return invalid_commit_marker_field("source_commit_lsn", "must be greater than zero");
    }
    if marker.global_event_count == 0 {
        return invalid_commit_marker_field("global_event_count", "must be greater than zero");
    }
    if marker.source_commit_timestamp_ms <= 0 {
        return invalid_commit_marker_field(
            "source_commit_timestamp_ms",
            "must be greater than zero",
        );
    }
    if marker.participating_partition_count == 0 {
        return invalid_commit_marker_field(
            "participating_partition_count",
            "must be greater than zero",
        );
    }
    if marker.manifest_checksum == 0 {
        return invalid_commit_marker_field("manifest_checksum", "must be greater than zero");
    }
    Ok(())
}

fn require_commit_marker_field(field: &'static str, value: &str) -> Result<(), ProtocolError> {
    if value.trim().is_empty() {
        return invalid_commit_marker_field(field, "must not be empty");
    }
    if value != value.trim() {
        return invalid_commit_marker_field(field, "must not contain surrounding whitespace");
    }
    Ok(())
}

fn invalid_commit_marker_field<T>(
    field: &'static str,
    reason: impl Into<String>,
) -> Result<T, ProtocolError> {
    Err(ProtocolError::InvalidCommitMarkerField {
        field,
        reason: reason.into(),
    })
}

fn participating_partition_count(manifest: &TransactionManifest) -> Result<u32, ProtocolError> {
    participating_partition_count_for_len(&manifest.transaction_id, manifest.partitions.len())
}

fn participating_partition_count_for_len(
    transaction_id: &str,
    partition_count: usize,
) -> Result<u32, ProtocolError> {
    u32::try_from(partition_count).map_err(|_| ProtocolError::ManifestCountOverflow {
        transaction_id: transaction_id.to_string(),
        field: "participating_partition_count",
        max_supported_count: u32::MAX,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn participating_partition_count_rejects_overflow() {
        let error = participating_partition_count_for_len("tx-count", u32::MAX as usize + 1)
            .expect_err("overflow");

        assert!(matches!(
            error,
            ProtocolError::ManifestCountOverflow {
                transaction_id,
                field: "participating_partition_count",
                max_supported_count: u32::MAX,
            } if transaction_id == "tx-count"
        ));
    }
}
