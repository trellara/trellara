use super::*;

#[test]
fn ddl_barrier_summary_rejects_target_ack_without_statement_evidence() {
    let summary = target_postgres_ack_summary("accepted");

    assert!(!summary.release_dml);
    assert_eq!(summary.rejected_sinks, vec!["target_postgres"]);
    assert_eq!(
        summary.release_blocker_codes,
        vec!["insufficient_target_postgres_evidence"]
    );
    let evidence = summary
        .sink_evidence
        .iter()
        .find(|evidence| evidence.sink == "target_postgres")
        .expect("target evidence");
    assert_eq!(
        evidence.rejection_code.as_deref(),
        Some("insufficient_target_postgres_evidence")
    );
    assert!(evidence
        .rejection_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("applied DDL statement evidence")));
}

#[test]
fn ddl_barrier_summary_rejects_target_ack_with_zero_applied_statements() {
    let summary = target_postgres_ack_summary(&target_postgres_ddl_ack_detail(0));

    assert_target_postgres_evidence_is_insufficient(&summary);
}

#[test]
fn ddl_barrier_summary_rejects_target_ack_with_spoofed_release_gate_detail() {
    let summary = target_postgres_ack_summary(
        "target Postgres applied 1 DDL statements; previous_release_gate=post_ddl_dml_release",
    );

    assert_target_postgres_evidence_is_insufficient(&summary);
}

#[test]
fn ddl_barrier_summary_rejects_target_ack_bound_to_different_barrier_lsn() {
    let digest = "a".repeat(64);
    let detail = target_postgres_ddl_ack_detail_with_boundary(
        1,
        &digest,
        std::slice::from_ref(&digest),
        Some("0/16C8000"),
    );
    let summary = target_postgres_ack_summary(&detail);

    assert!(!summary.release_dml);
    assert_eq!(summary.rejected_sinks, vec!["target_postgres"]);
    assert_eq!(
        summary.release_blocker_codes,
        vec!["target_postgres_barrier_lsn_mismatch"]
    );
    let evidence = summary
        .sink_evidence
        .iter()
        .find(|evidence| evidence.sink == "target_postgres")
        .expect("target evidence");
    assert_eq!(
        evidence.rejection_code.as_deref(),
        Some("target_postgres_barrier_lsn_mismatch")
    );
    assert!(evidence
        .rejection_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("must match the DDL barrier LSN")));
    assert!(!evidence.release_eligible);
}

fn target_postgres_ack_summary(detail: &str) -> DdlBarrierSummary {
    DdlBarrierSummary::from_barrier_and_acks(
        DdlBarrier {
            source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-target-proof".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
            required_sinks: vec!["target_postgres".to_string()],
            requires_global_partition_pause: false,
        },
        vec![DdlBarrierAck {
            source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-target-proof".to_string(),
            sink: "target_postgres".to_string(),
            ack_lsn: "0/16B9000".to_string(),
            schema_version: "schema-v2".to_string(),
            accepted: true,
            detail: detail.to_string(),
        }],
    )
}

fn assert_target_postgres_evidence_is_insufficient(summary: &DdlBarrierSummary) {
    assert!(!summary.release_dml);
    assert_eq!(summary.rejected_sinks, vec!["target_postgres"]);
    assert_eq!(
        summary.release_blocker_codes,
        vec!["insufficient_target_postgres_evidence"]
    );
    let evidence = summary
        .sink_evidence
        .iter()
        .find(|evidence| evidence.sink == "target_postgres")
        .expect("target evidence");
    assert_eq!(
        evidence.rejection_code.as_deref(),
        Some("insufficient_target_postgres_evidence")
    );
    assert!(!evidence.release_eligible);
}
