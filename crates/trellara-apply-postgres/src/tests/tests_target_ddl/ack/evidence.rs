use super::super::*;

#[test]
fn target_ddl_apply_outcome_builds_target_ack_evidence() {
    let evidence = TargetDdlApplyOutcome {
        barrier_id: "ddl-barrier-123".to_string(),
        applied_statements: 2,
        release_gate: "post_ddl_dml_release".to_string(),
        plan_sha256: "a".repeat(64),
        statement_sha256s: vec!["b".repeat(64), "c".repeat(64)],
    }
    .target_ack_evidence_for_database("source-a", "retail", "dataset-a", "0/16B9000", "schema-v2")
    .expect("ack evidence");

    assert_eq!(evidence.sink, "target_postgres");
    assert_eq!(evidence.applied_statements, 2);
    assert_eq!(evidence.release_gate, "post_ddl_dml_release");
    assert_eq!(evidence.barrier_lsn, None);
    assert_eq!(evidence.plan_sha256, "a".repeat(64));
    assert_eq!(evidence.statement_sha256s.len(), 2);

    let ack = evidence.into_barrier_ack();
    assert_eq!(ack.source_id, "source-a");
    assert_eq!(ack.database_id, "retail");
    assert_eq!(ack.dataset_id, "dataset-a");
    assert_eq!(ack.barrier_id, "ddl-barrier-123");
    assert_eq!(ack.sink, "target_postgres");
    assert_eq!(ack.ack_lsn, "0/16B9000");
    assert_eq!(ack.schema_version, "schema-v2");
    assert!(ack.accepted);
    assert!(ack.detail.contains("2 DDL statements"));
    assert!(ack.detail.contains("plan_sha256="));
    assert!(ack.detail.contains("statement_sha256="));
    assert!(ack.detail.contains("post_ddl_dml_release"));
}

#[test]
fn target_ddl_ack_evidence_canonicalizes_ack_lsn() {
    let evidence = TargetDdlApplyOutcome {
        barrier_id: "ddl-barrier-123".to_string(),
        applied_statements: 1,
        release_gate: "post_ddl_dml_release".to_string(),
        plan_sha256: "a".repeat(64),
        statement_sha256s: vec!["b".repeat(64)],
    }
    .target_ack_evidence_for_database(
        "source-a",
        "retail",
        "dataset-a",
        "00000000/016B9000",
        "schema-v2",
    )
    .expect("ack evidence");

    assert_eq!(evidence.ack_lsn, "0/16B9000");
    assert_eq!(evidence.into_barrier_ack().ack_lsn, "0/16B9000");
}
