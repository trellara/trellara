use super::*;
use trellara_protocol::{DdlEventInput, DdlOperation, RelationId, RelationSchemaVersion};

#[test]
fn strict_message_contains_envelope_headers_and_key() {
    let envelope = sample_envelope();
    let message = StreamMessage::strict_transaction(&envelope).expect("message");

    assert_eq!(message.topic, "trellara.source_a.sales.strict");
    assert_eq!(message.partition, Some(0));
    assert_eq!(message.position, None);
    assert_eq!(message.key, "source_a:retail:sales:tx-1:0/16B6C50");
    assert!(message
        .headers
        .contains(&StreamHeader::new("trellara.commit_lsn", "0/16B6C50")));
    assert!(message
        .headers
        .contains(&StreamHeader::new("trellara.schema_version_count", "0")));
    assert!(message
        .headers
        .contains(&StreamHeader::new("trellara.ddl_event_count", "0")));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.partitioned_scale_decision",
        "partition_parallel_dml"
    )));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.partition_parallel_safe",
        "true"
    )));
    assert!(message
        .headers
        .contains(&StreamHeader::new("trellara.requires_ddl_barrier", "false")));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.dml_replay_after_ddl_barrier_required",
        "false"
    )));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.partitioned_scale_reason",
        "DML-only transaction can be partitioned without a DDL barrier"
    )));
    assert!(!message
        .headers
        .iter()
        .any(|header| header.key == "trellara.schema_versions"));
    assert!(!message
        .headers
        .iter()
        .any(|header| header.key == "trellara.ddl_release_gates"));
    assert!(!message.payload.is_empty());
}

#[test]
fn envelope_headers_include_sorted_schema_versions() {
    let mut envelope = sample_envelope();
    envelope.schema_versions = vec![
        RelationSchemaVersion {
            relation: Some(RelationId::new(43, "public", "payments")),
            version: 3,
        },
        RelationSchemaVersion {
            relation: Some(RelationId::new(42, "public", "sales")),
            version: 7,
        },
    ];
    envelope.finalize_checksum();

    let message = StreamMessage::strict_transaction(&envelope).expect("message");

    assert!(message
        .headers
        .contains(&StreamHeader::new("trellara.schema_version_count", "2")));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.schema_versions",
        "public.payments=3,public.sales=7"
    )));
}

#[test]
fn envelope_headers_include_ddl_event_count_and_release_gates() {
    let mut envelope = sample_envelope();
    envelope.ddl_events = vec![DdlEvent::additive_column(
        "tx-1",
        2,
        RelationId::new(42, "public", "sales"),
        "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"discount_code\" text;",
        12_345,
        67_890,
    )];
    envelope.finalize_checksum();

    let message = StreamMessage::strict_transaction(&envelope).expect("message");

    assert!(message
        .headers
        .contains(&StreamHeader::new("trellara.ddl_event_count", "1")));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.partitioned_scale_decision",
        "ddl_barrier_required"
    )));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.partition_parallel_safe",
        "false"
    )));
    assert!(message
        .headers
        .contains(&StreamHeader::new("trellara.requires_ddl_barrier", "true")));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.dml_replay_after_ddl_barrier_required",
        "true"
    )));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.partitioned_scale_reason",
        "mixed DDL and DML transaction must apply DDL barrier before partitioned DML replay"
    )));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.ddl_release_gates",
        "post_ddl_dml_release"
    )));
    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.ddl_propagation_decisions",
        "propagation_decisions=auto_apply:1,manual_review:0,unsupported:0,target_ack_required:1"
    )));
    assert!(message
        .headers
        .contains(&StreamHeader::new("trellara.ddl_target_ack_required", "1")));
    assert!(message.headers.iter().any(|header| header.key
        == "trellara.ddl_propagation_policy_sha256"
        && header.value.len() == 64
        && header
            .value
            .chars()
            .all(|character| character.is_ascii_hexdigit())));
}

#[test]
fn envelope_headers_deduplicate_and_sort_ddl_release_gates() {
    let relation = RelationId::new(42, "public", "sales");
    let mut envelope = sample_envelope();
    envelope.ddl_events = vec![
        DdlEvent::classified(DdlEventInput {
            transaction_id: "tx-1".to_string(),
            total_order: 2,
            operation: DdlOperation::Other,
            relation: relation.clone(),
            statement: "ALTER TABLE public.sales VALIDATE CONSTRAINT sales_check;".to_string(),
            schema_fingerprint_before: 12_345,
            schema_fingerprint_after: 67_890,
            target_auto_apply: false,
            release_gate: "manual_gate".to_string(),
        }),
        DdlEvent::manual_review(
            "tx-1",
            3,
            DdlOperation::RenameColumn,
            relation.clone(),
            "ALTER TABLE public.sales RENAME COLUMN old_id TO new_id;",
            67_890,
            67_891,
        ),
        DdlEvent::additive_column(
            "tx-1",
            4,
            relation,
            "ALTER TABLE public.sales ADD COLUMN discount_code text;",
            67_891,
            67_892,
        ),
    ];
    envelope.finalize_checksum();

    let message = StreamMessage::strict_transaction(&envelope).expect("message");

    assert!(message.headers.contains(&StreamHeader::new(
        "trellara.ddl_release_gates",
        "manual_gate,post_ddl_dml_release"
    )));
    assert!(!message
        .headers
        .iter()
        .any(|header| header.key == "trellara.ddl_propagation_decisions"));
    assert!(!message
        .headers
        .iter()
        .any(|header| header.key == "trellara.ddl_target_ack_required"));
}
