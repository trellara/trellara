use trellara_protocol::{
    validate_transaction_manifest, ManifestBoundaryMode, TransactionEnvelope, TransactionManifest,
};

use crate::{
    protocol_transaction_boundary_kind_label, transaction_boundary_guarantee,
    transaction_visibility_contract, ChecksumStatus, TransactionBoundaryStatus,
    TransactionInspectBoundaryProof,
};

impl TransactionInspectBoundaryProof {
    pub(crate) fn from_envelope(envelope: &TransactionEnvelope) -> Self {
        let checksum_status = checksum_status(envelope);
        let manifest = envelope.manifest.as_ref();
        let global_event_count_matches =
            manifest.map(|manifest| manifest.global_event_count as usize == envelope.changes.len());
        let partition_event_count_matches =
            manifest.map(|manifest| partition_event_count_matches(manifest, envelope));
        let manifest_validation_error = manifest
            .and_then(|manifest| validate_transaction_manifest(manifest).err())
            .map(|error| error.to_string());
        let manifest_valid = manifest_validation_error.is_none();
        let manifest_counts_match = global_event_count_matches.unwrap_or(true)
            && partition_event_count_matches.unwrap_or(true);
        let status = boundary_status(checksum_status, manifest_counts_match, manifest_valid);
        let mode = boundary_mode(manifest);
        let partitioned_scale_readiness = envelope.partitioned_scale_readiness();
        let readiness_evidence = partitioned_scale_readiness.manifest_evidence.as_ref();

        Self {
            mode: mode.to_string(),
            source_boundary_kind: protocol_transaction_boundary_kind_label(
                partitioned_scale_readiness.boundary_kind,
            )
            .to_string(),
            partitioned_scale_decision: partitioned_scale_readiness.decision.to_string(),
            partition_parallel_safe: partitioned_scale_readiness.partition_parallel_safe,
            requires_ddl_barrier: partitioned_scale_readiness.requires_ddl_barrier,
            dml_replay_after_ddl_barrier_required: partitioned_scale_readiness
                .dml_replay_after_ddl_barrier_required,
            partitioned_scale_reason: partitioned_scale_readiness.reason.to_string(),
            status,
            guarantee: transaction_boundary_guarantee(mode).to_string(),
            visibility_contract: transaction_visibility_contract(mode).to_string(),
            checksum_status,
            manifest_barrier_required: manifest.is_some(),
            manifest_valid,
            manifest_validation_error,
            global_event_count_matches,
            partition_event_count_matches,
            partitioned_scale_manifest_checksum: readiness_evidence
                .map(|evidence| evidence.manifest_checksum),
            partitioned_scale_manifest_event_count: readiness_evidence
                .map(|evidence| evidence.manifest_event_count),
            partitioned_scale_envelope_event_count: readiness_evidence
                .map(|evidence| evidence.envelope_event_count),
            partitioned_scale_event_count_coverage: readiness_evidence
                .map(|evidence| evidence.event_count_coverage),
            partitioned_scale_participating_partition_ids: readiness_evidence
                .map(|evidence| evidence.participating_partition_ids.clone())
                .unwrap_or_default(),
            partitioned_scale_visibility_contract: readiness_evidence
                .map(|evidence| evidence.visibility_contract.to_string()),
            participating_partition_count: manifest
                .map(|manifest| manifest.partitions.len())
                .unwrap_or_default(),
            expected_commit_marker_manifest_checksum: manifest
                .map(TransactionManifest::compute_checksum),
        }
    }
}

fn checksum_status(envelope: &TransactionEnvelope) -> ChecksumStatus {
    if envelope.verify_checksum().is_ok() {
        ChecksumStatus::Match
    } else {
        ChecksumStatus::Mismatch
    }
}

fn partition_event_count_matches(
    manifest: &TransactionManifest,
    envelope: &TransactionEnvelope,
) -> bool {
    manifest
        .partitions
        .iter()
        .map(|partition| partition.event_count as usize)
        .sum::<usize>()
        == envelope.changes.len()
}

fn boundary_status(
    checksum_status: ChecksumStatus,
    manifest_counts_match: bool,
    manifest_valid: bool,
) -> TransactionBoundaryStatus {
    if checksum_status == ChecksumStatus::Match && manifest_counts_match && manifest_valid {
        TransactionBoundaryStatus::Verified
    } else {
        TransactionBoundaryStatus::AtRisk
    }
}

fn boundary_mode(manifest: Option<&TransactionManifest>) -> &'static str {
    manifest
        .map(|manifest| {
            ManifestBoundaryMode::try_from(manifest.boundary_mode)
                .unwrap_or(ManifestBoundaryMode::Unspecified)
                .status_mode()
        })
        .unwrap_or("strict_transaction_order")
}
