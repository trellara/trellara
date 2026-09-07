use trellara_protocol::{
    PartitionedScaleManifestEvidence, TransactionEnvelope, TransactionManifest,
};

use crate::headers::envelope_headers;
use crate::StreamHeader;

pub fn manifest_headers(
    envelope: &TransactionEnvelope,
    manifest: &TransactionManifest,
) -> Vec<StreamHeader> {
    let mut headers = envelope_headers(envelope);
    let evidence = PartitionedScaleManifestEvidence::from_manifest(manifest, envelope);
    headers.push(StreamHeader::new("trellara.message_kind", "manifest"));
    headers.push(StreamHeader::new(
        "trellara.global_event_count",
        manifest.global_event_count.to_string(),
    ));
    headers.push(StreamHeader::new(
        "trellara.partition_count",
        manifest.partitions.len().to_string(),
    ));
    headers.push(StreamHeader::new(
        "trellara.partitioned_scale_manifest_checksum",
        evidence.manifest_checksum.to_string(),
    ));
    headers.push(StreamHeader::new(
        "trellara.partitioned_scale_manifest_event_count",
        evidence.manifest_event_count.to_string(),
    ));
    headers.push(StreamHeader::new(
        "trellara.partitioned_scale_envelope_event_count",
        evidence.envelope_event_count.to_string(),
    ));
    headers.push(StreamHeader::new(
        "trellara.partitioned_scale_event_count_coverage",
        evidence.event_count_coverage.to_string(),
    ));
    headers.push(StreamHeader::new(
        "trellara.partitioned_scale_participating_partition_ids",
        evidence
            .participating_partition_ids
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(","),
    ));
    headers.push(StreamHeader::new(
        "trellara.partitioned_scale_visibility_contract",
        evidence.visibility_contract,
    ));
    headers
}
