use super::fixtures::{ddl_release_action, ddl_release_blocker, ddl_release_gate};
use super::*;

#[test]
fn ddl_barrier_text_summary_explains_target_proof_rejection() {
    let summary = DdlBarrierSummary {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        barrier_id: "ddl-barrier-target-proof".to_string(),
        barrier_lsn: "0/16B8000".to_string(),
        schema_version: "schema-v2".to_string(),
        cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
        propagation_boundary: None,
        propagation_decisions: Vec::new(),
        propagation_policy_sha256: None,
        required_sink_count: 1,
        acked_sink_count: 0,
        pending_sink_count: 0,
        rejected_sink_count: 1,
        unexpected_ack_count: 0,
        acked_sinks: Vec::new(),
        pending_sinks: Vec::new(),
        rejected_sinks: vec!["target_postgres".to_string()],
        unexpected_sinks: Vec::new(),
        sink_evidence: vec![DdlBarrierSinkEvidence {
            sink: "target_postgres".to_string(),
            status: "rejected".to_string(),
            ack_lsn: Some("0/16B9000".to_string()),
            schema_version: Some("schema-v2".to_string()),
            accepted: Some(true),
            detail: Some("accepted".to_string()),
            release_eligible: false,
            rejection_code: Some("insufficient_target_postgres_evidence".to_string()),
            rejection_reason: Some(
                "target_postgres ACK detail must include applied DDL statement evidence"
                    .to_string(),
            ),
        }],
        release_blockers: vec![
            "rejected or stale sink acknowledgements: target_postgres".to_string()
        ],
        release_blocker_codes: vec!["insufficient_target_postgres_evidence".to_string()],
        release_blocker_details: vec![ddl_release_blocker(
            "rejected_or_stale_ack",
            "rejected or stale sink acknowledgements",
            &["target_postgres"],
        )],
        release_actions: vec![ddl_release_action(
            "replace_rejected_sink_ack",
            &["target_postgres"],
        )],
        release_gates: vec![ddl_release_gate(
            "post_ddl_dml_release",
            false,
            "release_dml is false",
        )],
        release_dml: false,
        requires_global_partition_pause: false,
    };

    let output =
        render_ddl_barrier_summary(&summary, QuickstartOutputFormat::Text).expect("render");

    assert!(output.contains("release_blocker_codes: insufficient_target_postgres_evidence"));
    assert!(output
        .contains("reason=target_postgres ACK detail must include applied DDL statement evidence"));
    assert!(output.contains(
        "rerun target_postgres DDL apply and record ACK detail containing applied DDL statements, plan_sha256, statement_sha256 values, and release_gate=post_ddl_dml_release"
    ));
}
