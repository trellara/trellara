use super::fixtures::{
    ddl_release_action, ddl_release_blocker, ddl_release_gate, ddl_sink_evidence,
};
use super::*;

#[test]
fn ddl_barrier_text_summary_names_release_and_pending_sinks() {
    let summary = DdlBarrierSummary {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        barrier_id: "ddl-barrier-abc".to_string(),
        barrier_lsn: "0/16B8000".to_string(),
        schema_version: "schema-v2".to_string(),
        cdc_transaction_boundary: "source commit LSN is the DDL barrier; propagation_boundary=source_commit_lsn_barrier_holds_post_ddl_dml_until_required_sink_acks; propagation_decisions=auto_apply:1,manual_review:1,unsupported:0,target_ack_required:2; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
        propagation_boundary: Some(
            "source_commit_lsn_barrier_holds_post_ddl_dml_until_required_sink_acks".to_string(),
        ),
        propagation_decisions: vec![
            "auto_apply:1".to_string(),
            "manual_review:1".to_string(),
            "unsupported:0".to_string(),
            "target_ack_required:2".to_string(),
        ],
        propagation_policy_sha256: Some("a".repeat(64)),
        required_sink_count: 3,
        acked_sink_count: 1,
        pending_sink_count: 2,
        rejected_sink_count: 0,
        unexpected_ack_count: 1,
        acked_sinks: vec!["target_postgres".to_string()],
        pending_sinks: vec![
            "raw_cdc_lake".to_string(),
            "spark_derived_views".to_string(),
        ],
        rejected_sinks: Vec::new(),
        unexpected_sinks: vec!["typo_target_postgres".to_string()],
        sink_evidence: vec![
            ddl_sink_evidence("target_postgres", "acked", Some("0/16B9000"), true),
            ddl_sink_evidence("raw_cdc_lake", "pending", None, false),
            ddl_sink_evidence("spark_derived_views", "pending", None, false),
            ddl_sink_evidence(
                "typo_target_postgres",
                "unexpected",
                Some("0/16B9000"),
                false,
            ),
        ],
        release_blockers: vec![
            "pending required sink acknowledgements: raw_cdc_lake, spark_derived_views".to_string(),
            "unexpected sink acknowledgements: typo_target_postgres".to_string(),
        ],
        release_blocker_codes: vec![
            "ack_lsn_before_barrier".to_string(),
            "pending_required_ack".to_string(),
            "partition_visibility_not_released".to_string(),
            "unexpected_sink_ack".to_string(),
        ],
        release_blocker_details: vec![
            ddl_release_blocker(
                "pending_required_ack",
                "pending required sink acknowledgements",
                &["raw_cdc_lake", "spark_derived_views"],
            ),
            ddl_release_blocker(
                "unexpected_ack",
                "unexpected sink acknowledgements",
                &["typo_target_postgres"],
            ),
        ],
        release_actions: vec![
            ddl_release_action(
                "record_required_sink_ack",
                &["raw_cdc_lake", "spark_derived_views"],
            ),
            ddl_release_action("remove_unexpected_sink_ack", &["typo_target_postgres"]),
        ],
        release_gates: vec![
            ddl_release_gate("schema_barrier_recorded", true, "persisted"),
            ddl_release_gate("required_sink_acknowledgements", false, "blocked"),
            ddl_release_gate("partition_visibility_watermark", false, "blocked"),
            ddl_release_gate("post_ddl_dml_release", false, "release_dml is false"),
        ],
        release_dml: false,
        requires_global_partition_pause: true,
    };

    let output =
        render_ddl_barrier_summary(&summary, QuickstartOutputFormat::Text).expect("render");

    assert!(output.contains("Trellara DDL barrier status"));
    assert!(output.contains("dataset: sales source=source-a database=retail"));
    assert!(output.contains("propagation_boundary: source_commit_lsn_barrier_holds_post_ddl_dml_until_required_sink_acks"));
    assert!(output.contains(
        "propagation_decisions: auto_apply:1, manual_review:1, unsupported:0, target_ack_required:2"
    ));
    assert!(output.contains(&format!("propagation_policy_sha256: {}", "a".repeat(64))));
    assert!(output.contains(
        "release_dml: false required_sinks=3 acked=1 pending=2 rejected=0 unexpected=1 partition_pause=true"
    ));
    assert!(output.contains("acked_sinks: target_postgres"));
    assert!(output.contains("pending_sinks: raw_cdc_lake, spark_derived_views"));
    assert!(output.contains("rejected_sinks: none"));
    assert!(output.contains("unexpected_sinks: typo_target_postgres count=1"));
    assert!(output.contains(
        "release_blockers: pending required sink acknowledgements: raw_cdc_lake, spark_derived_views, unexpected sink acknowledgements: typo_target_postgres"
    ));
    assert!(output.contains(
        "release_blocker_codes: ack_lsn_before_barrier, pending_required_ack, partition_visibility_not_released, unexpected_sink_ack"
    ));
    assert!(output.contains("release_blocker_details:"));
    assert!(output.contains(
        "code=pending_required_ack sinks=raw_cdc_lake, spark_derived_views message=pending required sink acknowledgements evidence=barrier_id=ddl-barrier-abc barrier_lsn=0/16B8000 schema_version=schema-v2"
    ));
    assert!(output.contains(
        "code=unexpected_ack sinks=typo_target_postgres message=unexpected sink acknowledgements evidence=barrier_id=ddl-barrier-abc barrier_lsn=0/16B8000 schema_version=schema-v2"
    ));
    assert!(output.contains("release_gates:"));
    assert!(output.contains("schema_barrier_recorded satisfied=true evidence=persisted"));
    assert!(output.contains("post_ddl_dml_release satisfied=false evidence=release_dml is false"));
    assert!(output.contains("sink_evidence:"));
    assert!(output.contains(
        "target_postgres status=acked release_eligible=true ack_lsn=0/16B9000 schema_version=schema-v2 accepted=true rejection_code=none reason=none"
    ));
    assert!(output.contains("  detail: target schema fingerprint matched"));
    assert!(output.contains("raw_cdc_lake status=pending release_eligible=false ack_lsn=none"));
    assert!(output.contains(
        "typo_target_postgres status=unexpected release_eligible=false ack_lsn=0/16B9000"
    ));
    assert!(output.contains("next_actions:"));
    assert!(output.contains(
        "record required ACKs for pending sinks raw_cdc_lake, spark_derived_views on barrier ddl-barrier-abc with ack_lsn at or beyond 0/16B8000 and schema_version schema-v2"
    ));
    assert!(output.contains(
        "remove or investigate unexpected sink ACKs before release: typo_target_postgres"
    ));
    assert!(output.contains(
        "capture durable sink ACK evidence for raw_cdc_lake, spark_derived_views using barrier_id=ddl-barrier-abc barrier_lsn=0/16B8000 schema_version=schema-v2"
    ));
    assert!(output.contains(
        "remove ACK records for non-required sinks typo_target_postgres or add them to the barrier required_sinks contract; blocker evidence: barrier_id=ddl-barrier-abc barrier_lsn=0/16B8000 schema_version=schema-v2"
    ));
    assert!(output.contains(
        "wait for sinks to reach barrier_lsn 0/16B8000, then record a fresh ACK for barrier ddl-barrier-abc"
    ));
    assert!(output.contains(
        "run partition-watermarks for barrier ddl-barrier-abc, verify every partition lane reached barrier_lsn 0/16B8000, then record the partition_visibility ACK with --barrier-lsn 0/16B8000 --schema-version schema-v2, --partition-durable-lsn for each lane, and --partition-applied-lsn for each lane"
    ));
}
