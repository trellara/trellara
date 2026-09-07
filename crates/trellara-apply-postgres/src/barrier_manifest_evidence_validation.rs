use trellara_protocol::{PartitionedScaleManifestEvidence, TransactionManifest};

use crate::barrier_header_context::HeaderContext;
use crate::barrier_header_payload::validate_header_payload_field;
use crate::ApplyWorkerResult;

pub(crate) fn validate_manifest_evidence_header_context(
    context: &HeaderContext,
    manifest: &TransactionManifest,
) -> ApplyWorkerResult<()> {
    let evidence = PartitionedScaleManifestEvidence::from_manifest_counts(
        manifest,
        manifest.global_event_count as usize,
    );
    validate_header_payload_field(
        "partitioned_scale_manifest_checksum",
        &context.manifest_evidence.manifest_checksum,
        &evidence.manifest_checksum.to_string(),
    )?;
    validate_header_payload_field(
        "partitioned_scale_manifest_event_count",
        &context.manifest_evidence.manifest_event_count,
        &evidence.manifest_event_count.to_string(),
    )?;
    validate_header_payload_field(
        "partitioned_scale_envelope_event_count",
        &context.manifest_evidence.envelope_event_count,
        &manifest.global_event_count.to_string(),
    )?;
    validate_header_payload_field(
        "partitioned_scale_event_count_coverage",
        &context.manifest_evidence.event_count_coverage,
        &evidence.event_count_coverage.to_string(),
    )?;
    validate_header_payload_field(
        "partitioned_scale_participating_partition_ids",
        &context.manifest_evidence.participating_partition_ids,
        &evidence
            .participating_partition_ids
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(","),
    )?;
    validate_header_payload_field(
        "partitioned_scale_visibility_contract",
        &context.manifest_evidence.visibility_contract,
        evidence.visibility_contract,
    )
}
