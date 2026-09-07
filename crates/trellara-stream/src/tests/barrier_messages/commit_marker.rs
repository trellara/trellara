use super::helpers::{assert_barrier_payload_mismatch, commit_marker, partition_plan};
use super::*;

#[test]
fn commit_marker_message_uses_commit_topic_and_headers() {
    let envelope = sample_envelope();
    let plan = partition_plan();
    let marker = commit_marker(&plan);

    let message = StreamMessage::commit_marker(&envelope, &marker).expect("commit marker");

    assert_eq!(message.topic, "trellara.source_a.sales.commit");
    assert_eq!(message.partition, Some(0));
    assert_eq!(message.key, "source_a:retail:sales:tx-1:0/16B6C50");
    assert!(message
        .headers
        .contains(&StreamHeader::new("trellara.message_kind", "commit_marker")));
    assert!(message
        .headers
        .contains(&StreamHeader::new("trellara.partition_count", "1")));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.partitioned_scale_decision",
        "partition_parallel_dml"
    )));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.partitioned_scale_reason",
        "DML-only transaction can be partitioned without a DDL barrier"
    )));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.manifest_checksum",
        marker.manifest_checksum.to_string()
    )));
    let decoded = TransactionCommitMarker::decode(message.payload.as_ref()).expect("decode marker");
    assert_eq!(decoded.transaction_id, "tx-1");
    assert_eq!(decoded.source_commit_lsn, "0/16B6C50");
    assert_eq!(decoded.manifest_checksum, plan.manifest.compute_checksum());
}

#[test]
fn commit_marker_message_rejects_envelope_boundary_mismatch() {
    let envelope = sample_envelope();
    let plan = partition_plan();
    let mut marker = commit_marker(&plan);
    marker.source_commit_lsn = "0/16B6C51".to_string();

    assert_barrier_payload_mismatch(
        StreamMessage::commit_marker(&envelope, &marker),
        "commit_lsn",
    );
}

#[test]
fn commit_marker_message_rejects_global_event_count_mismatch() {
    let envelope = sample_envelope();
    let plan = partition_plan();
    let mut marker = commit_marker(&plan);
    marker.global_event_count += 1;

    assert_barrier_payload_mismatch(
        StreamMessage::commit_marker(&envelope, &marker),
        "global_event_count",
    );
}

#[test]
fn commit_marker_message_rejects_zero_participating_partitions() {
    let envelope = sample_envelope();
    let plan = partition_plan();
    let mut marker = commit_marker(&plan);
    marker.participating_partition_count = 0;

    assert!(matches!(
        StreamMessage::commit_marker(&envelope, &marker),
        Err(StreamError::Protocol(
            ProtocolError::InvalidCommitMarkerField {
                field: "participating_partition_count",
                ..
            }
        ))
    ));
}
