use super::helpers::{assert_barrier_payload_mismatch, partition_plan};
use super::*;

#[test]
fn manifest_message_uses_barrier_topic_and_headers() {
    let envelope = sample_envelope();
    let plan = partition_plan();

    let message = StreamMessage::transaction_manifest(&envelope, &plan.manifest).expect("manifest");

    assert_eq!(message.topic, "trellara.source_a.sales.manifest");
    assert_eq!(message.partition, Some(0));
    assert_eq!(message.key, "source_a:retail:sales:tx-1:0/16B6C50");
    assert!(message
        .headers
        .contains(&StreamHeader::new("trellara.message_kind", "manifest")));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.partitioned_scale_decision",
        "partition_parallel_dml"
    )));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.partition_parallel_safe",
        "true"
    )));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.partitioned_scale_manifest_checksum",
        plan.manifest.compute_checksum().to_string()
    )));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.partitioned_scale_manifest_event_count",
        "1"
    )));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.partitioned_scale_envelope_event_count",
        "1"
    )));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.partitioned_scale_event_count_coverage",
        "true"
    )));
    assert!(message.headers.iter().any(|header| {
        header.key == "trellara.partitioned_scale_participating_partition_ids"
            && header.value == plan.manifest.partitions[0].id.to_string()
    }));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.partitioned_scale_visibility_contract",
        "global visibility waits for manifest, commit marker, and every participating partition"
    )));
    let decoded = TransactionManifest::decode(message.payload.as_ref()).expect("decode");
    assert_eq!(decoded.transaction_id, "tx-1");
    assert_eq!(decoded.global_event_count, 1);
}

#[test]
fn manifest_message_rejects_envelope_boundary_mismatch() {
    let envelope = sample_envelope();
    let mut plan = partition_plan();
    plan.manifest.source_commit_lsn = "0/16B6C51".to_string();

    assert_barrier_payload_mismatch(
        StreamMessage::transaction_manifest(&envelope, &plan.manifest),
        "commit_lsn",
    );
}

#[test]
fn manifest_message_rejects_global_event_count_mismatch() {
    let envelope = sample_envelope();
    let mut plan = partition_plan();
    plan.manifest.global_event_count += 1;

    assert_barrier_payload_mismatch(
        StreamMessage::transaction_manifest(&envelope, &plan.manifest),
        "global_event_count",
    );
}

#[test]
fn manifest_message_rejects_affected_table_count_mismatch() {
    let envelope = sample_envelope();
    let mut plan = partition_plan();
    plan.manifest.affected_tables[0].event_count += 1;

    assert!(matches!(
        StreamMessage::transaction_manifest(&envelope, &plan.manifest),
        Err(StreamError::Protocol(
            ProtocolError::ManifestAffectedTableCountMismatch { .. }
        ))
    ));
}
