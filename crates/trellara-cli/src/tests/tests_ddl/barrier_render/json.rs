use super::fixtures::{
    ddl_rejected_sink_evidence, ddl_release_action, ddl_release_blocker, ddl_release_gate,
    ddl_sink_evidence,
};
use super::*;

#[test]
fn ddl_barrier_json_summary_exposes_release_gate() {
    let summary = DdlBarrierSummary {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        barrier_id: "ddl-barrier-abc".to_string(),
        barrier_lsn: "0/16B8000".to_string(),
        schema_version: "schema-v2".to_string(),
        cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
        propagation_boundary: None,
        propagation_decisions: Vec::new(),
        propagation_policy_sha256: None,
        required_sink_count: 2,
        acked_sink_count: 1,
        pending_sink_count: 0,
        rejected_sink_count: 1,
        unexpected_ack_count: 0,
        acked_sinks: vec!["target_postgres".to_string()],
        pending_sinks: Vec::new(),
        rejected_sinks: vec!["raw_cdc_lake".to_string()],
        unexpected_sinks: Vec::new(),
        sink_evidence: vec![
            ddl_sink_evidence("target_postgres", "acked", Some("0/16B9000"), true),
            ddl_rejected_sink_evidence("raw_cdc_lake", "schema_version_mismatch"),
        ],
        release_blockers: vec!["rejected or stale sink acknowledgements: raw_cdc_lake".to_string()],
        release_blocker_codes: vec!["schema_version_mismatch".to_string()],
        release_blocker_details: vec![ddl_release_blocker(
            "rejected_or_stale_ack",
            "rejected or stale sink acknowledgements",
            &["raw_cdc_lake"],
        )],
        release_actions: vec![ddl_release_action(
            "replace_rejected_sink_ack",
            &["raw_cdc_lake"],
        )],
        release_gates: vec![
            ddl_release_gate("schema_barrier_recorded", true, "persisted"),
            ddl_release_gate("post_ddl_dml_release", false, "release_dml is false"),
        ],
        release_dml: false,
        requires_global_partition_pause: false,
    };

    let output =
        render_ddl_barrier_summary(&summary, QuickstartOutputFormat::Json).expect("render");

    assert!(output.contains("\"release_dml\": false"));
    assert!(output.contains("\"database_id\": \"retail\""));
    assert!(
        output.contains("\"cdc_transaction_boundary\": \"source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn\"")
    );
    assert!(output.contains("\"acked_sink_count\": 1"));
    assert!(output.contains("\"pending_sink_count\": 0"));
    assert!(output.contains("\"rejected_sink_count\": 1"));
    assert!(output.contains("\"unexpected_ack_count\": 0"));
    assert!(output.contains("\"acked_sinks\""));
    assert!(output.contains("\"unexpected_sinks\""));
    assert!(output.contains("rejected or stale sink acknowledgements"));
    assert!(output.contains("\"release_blocker_codes\""));
    assert!(output.contains("\"release_blocker_details\""));
    assert!(output.contains("\"release_actions\""));
    assert!(output.contains("\"replace_rejected_sink_ack\""));
    assert!(output.contains("\"evidence\""));
    assert!(output.contains("barrier_lsn=0/16B8000"));
    assert!(output.contains("\"schema_version_mismatch\""));
    assert!(output.contains("\"rejected_or_stale_ack\""));
    assert!(output.contains("\"release_gates\""));
    assert!(output.contains("\"post_ddl_dml_release\""));
    assert!(output.contains("\"sink_evidence\""));
    assert!(output.contains("\"release_eligible\": true"));
    assert!(output.contains("\"rejection_code\": \"schema_version_mismatch\""));
    assert!(output.contains("\"accepted\": true"));
    assert!(output.contains("\"detail\": \"target schema fingerprint matched\""));
    assert!(output.contains("target_postgres"));
    assert!(output.contains("\"next_actions\""));
    assert!(output.contains(
        "replace rejected or stale ACK evidence for raw_cdc_lake before releasing post-DDL DML; blocker evidence: barrier_id=ddl-barrier-abc barrier_lsn=0/16B8000 schema_version=schema-v2"
    ));
    assert!(
        output.contains("refresh schema discovery and record ACKs with the barrier schema_version")
    );
}
