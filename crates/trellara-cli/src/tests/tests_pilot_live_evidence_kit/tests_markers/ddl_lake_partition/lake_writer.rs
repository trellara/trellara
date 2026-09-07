use super::*;

const VALID_WRITER_PLAN: &str = r#"{
    "dataset_id": "retail-sales",
    "duplicate_transaction_count": 1,
    "duplicate_replay_evidence": {
        "contract": "duplicate replay is skipped by transaction identity and conflicting idempotency evidence fails closed",
        "duplicate_transaction_count": 1,
        "unique_transaction_count": 1,
        "row_intent_count": 1,
        "idempotency_key_count": 1,
        "replay_safe": true
    },
    "committer_topology": {
        "source_ack_boundary": "source acknowledgement advances after durable Trellara stream publish, not after Iceberg catalog commit"
    },
    "commit_steps": [
        {
            "order": 10,
            "phase": "raw_cdc_data",
            "durability_gate": "before source acknowledgement"
        },
        {
            "order": 120,
            "phase": "epoch_row_metadata",
            "durability_gate": "all planned raw CDC data files and Iceberg checkpoint receipts for the epoch are durable before this metadata step"
        },
        {
            "order": 130,
            "phase": "verification_metadata",
            "durability_gate": "all planned raw CDC data files and Iceberg checkpoint receipts for the epoch are durable before this metadata step"
        }
    ],
    "recovery_scenarios": [
        {
            "replay_policy": "discover_existing_epoch_metadata_before_publish",
            "recovery_action": "verify checkpoint receipts and discover existing epoch metadata before publishing duplicate visibility"
        }
    ],
    "epoch_metadata": {
        "source_rows": [
            {"source_id": "local-source"}
        ],
        "partition_rows": [
            {"source_id": "local-source", "partition_id": 0}
        ]
    },
    "row_intents": [
        {
            "source_id": "local-source",
            "dataset_id": "retail-sales",
            "relation": "public.sales",
            "transaction_id": "tx-lake-writer-sample",
            "total_order": 2,
            "idempotency_key": "local-source:0/16B6C50:tx-lake-writer-sample:2",
            "schema_version": 67890,
            "ddl_barrier_id": "local-source:retail-sales:tx-lake-writer-sample:0/16B6C50:ddl",
            "ddl_release_gate": "post_ddl_dml_release",
            "ddl_schema_fingerprint_before": 12345,
            "ddl_schema_fingerprint_after": 67890
        }
    ]
}"#;

#[test]
fn lake_writer_plan_requires_transaction_and_ddl_boundary_markers() {
    assert_eq!(
        live_evidence_artifact_name("lake_writer_plan"),
        "lake-writer-plan.json"
    );

    for marker in live_evidence_expected_markers("lake_writer_plan") {
        assert!(
            live_evidence_marker_present("lake_writer_plan", &marker, VALID_WRITER_PLAN),
            "expected marker {marker} in lake writer plan"
        );
    }
}

#[test]
fn lake_writer_plan_rejects_partition_source_without_source_row_evidence() {
    let plan = VALID_WRITER_PLAN.replace(
        r#""source_id": "local-source", "partition_id": 0"#,
        r#""source_id": "missing-source", "partition_id": 0"#,
    );

    assert!(!live_evidence_marker_present(
        "lake_writer_plan",
        "partition source evidence",
        &plan
    ));
}

#[test]
fn lake_writer_plan_rejects_missing_ddl_boundary_metadata() {
    let plan = VALID_WRITER_PLAN.replace(r#""ddl_release_gate": "post_ddl_dml_release","#, "");

    assert!(!live_evidence_marker_present(
        "lake_writer_plan",
        "DDL boundary metadata",
        &plan
    ));
}

#[test]
fn lake_writer_plan_rejects_mixed_rows_without_ddl_boundary_metadata() {
    let plan = VALID_WRITER_PLAN.replace(
        r#""row_intents": ["#,
        r#""row_intents": [
        {
            "source_id": "local-source",
            "dataset_id": "retail-sales",
            "relation": "public.sales",
            "transaction_id": "tx-lake-writer-sample",
            "total_order": 1,
            "idempotency_key": "local-source:0/16B6C50:tx-lake-writer-sample:1"
        },"#,
    );

    assert!(!live_evidence_marker_present(
        "lake_writer_plan",
        "DDL boundary metadata",
        &plan
    ));
}

#[test]
fn lake_writer_plan_rejects_regressing_schema_fingerprint() {
    let plan = VALID_WRITER_PLAN.replace(
        r#""ddl_schema_fingerprint_after": 67890"#,
        r#""ddl_schema_fingerprint_after": 12345"#,
    );

    assert!(!live_evidence_marker_present(
        "lake_writer_plan",
        "DDL boundary metadata",
        &plan
    ));
}

#[test]
fn lake_writer_plan_rejects_unordered_commit_steps() {
    let plan = VALID_WRITER_PLAN.replace(
        r#""phase": "verification_metadata""#,
        r#""phase": "missing_verification""#,
    );

    assert!(!live_evidence_marker_present(
        "lake_writer_plan",
        "commit ordering",
        &plan
    ));
}

#[test]
fn lake_writer_plan_rejects_weak_source_ack_boundary() {
    let plan = VALID_WRITER_PLAN.replace(
        "source acknowledgement advances after durable Trellara stream publish, not after Iceberg catalog commit",
        "source acknowledgement advances after durable Trellara stream publish",
    );

    assert!(!live_evidence_marker_present(
        "lake_writer_plan",
        "durability gates",
        &plan
    ));
}

#[test]
fn lake_writer_plan_rejects_missing_replay_accounting() {
    let plan = VALID_WRITER_PLAN.replace(
        "discover_existing_epoch_metadata_before_publish",
        "operator_review_required",
    );

    assert!(!live_evidence_marker_present(
        "lake_writer_plan",
        "duplicate replay accounting",
        &plan
    ));
}

#[test]
fn lake_writer_plan_rejects_replay_accounting_that_disagrees_with_row_intents() {
    let row_count_mismatch =
        VALID_WRITER_PLAN.replace(r#""row_intent_count": 1"#, r#""row_intent_count": 2"#);
    let idempotency_count_mismatch = VALID_WRITER_PLAN.replace(
        r#""idempotency_key_count": 1"#,
        r#""idempotency_key_count": 2"#,
    );

    assert!(!live_evidence_marker_present(
        "lake_writer_plan",
        "duplicate replay accounting",
        &row_count_mismatch
    ));
    assert!(!live_evidence_marker_present(
        "lake_writer_plan",
        "duplicate replay accounting",
        &idempotency_count_mismatch
    ));
}

#[test]
fn lake_writer_plan_rejects_missing_structured_replay_evidence() {
    let plan = VALID_WRITER_PLAN.replace(
        r#"    "duplicate_replay_evidence": {
        "contract": "duplicate replay is skipped by transaction identity and conflicting idempotency evidence fails closed",
        "duplicate_transaction_count": 1,
        "unique_transaction_count": 1,
        "row_intent_count": 1,
        "idempotency_key_count": 1,
        "replay_safe": true
    },
"#,
        "",
    );

    assert!(!live_evidence_marker_present(
        "lake_writer_plan",
        "duplicate replay accounting",
        &plan
    ));
}

#[test]
fn lake_writer_plan_rejects_missing_checkpoint_receipt_gate() {
    let plan = VALID_WRITER_PLAN.replace(
        "all planned raw CDC data files and Iceberg checkpoint receipts for the epoch are durable before this metadata step",
        "all planned raw CDC data files for the epoch are durable before this metadata step",
    );

    assert!(!live_evidence_marker_present(
        "lake_writer_plan",
        "Iceberg checkpoint receipt gate",
        &plan
    ));
}
