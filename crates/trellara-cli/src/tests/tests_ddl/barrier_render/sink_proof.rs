use super::fixtures::{ddl_release_action, ddl_release_blocker, ddl_release_gate};
use super::*;

#[test]
fn ddl_barrier_text_summary_explains_raw_lake_proof_rejection() {
    let summary = rejected_sink_summary(
        "raw_cdc_lake",
        "insufficient_raw_cdc_lake_evidence",
        "raw_cdc_lake ACK detail must include schema_version, epoch metadata, partition metadata, manifest_digest, and release_gate=post_ddl_dml_release",
    );

    let output =
        render_ddl_barrier_summary(&summary, QuickstartOutputFormat::Text).expect("render");

    assert!(output.contains("release_blocker_codes: insufficient_raw_cdc_lake_evidence"));
    assert!(output.contains(
        "reason=raw_cdc_lake ACK detail must include schema_version, epoch metadata, partition metadata, manifest_digest, and release_gate=post_ddl_dml_release"
    ));
    assert!(output.contains(
        "rerun raw CDC lake epoch finalization for barrier ddl-barrier-sink-proof and record raw_cdc_lake ACK detail containing schema_version schema-v2, epoch_id, metadata_table, partition_metadata_table, manifest_digest, and release_gate=post_ddl_dml_release"
    ));
}

#[test]
fn ddl_barrier_text_summary_explains_spark_proof_rejection() {
    let summary = rejected_sink_summary(
        "spark_derived_views",
        "insufficient_spark_derived_views_evidence",
        "spark_derived_views ACK detail must include regenerated view count, template_digest, accepted_by, and release_gate=post_ddl_dml_release",
    );

    let output =
        render_ddl_barrier_summary(&summary, QuickstartOutputFormat::Text).expect("render");

    assert!(output.contains("release_blocker_codes: insufficient_spark_derived_views_evidence"));
    assert!(output.contains(
        "reason=spark_derived_views ACK detail must include regenerated view count, template_digest, accepted_by, and release_gate=post_ddl_dml_release"
    ));
    assert!(output.contains(
        "regenerate Spark derived view templates and record spark_derived_views ACK detail containing regenerated view_count, template_digest, accepted_by, and release_gate=post_ddl_dml_release"
    ));
}

fn rejected_sink_summary(sink: &str, code: &str, reason: &str) -> DdlBarrierSummary {
    DdlBarrierSummary {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        barrier_id: "ddl-barrier-sink-proof".to_string(),
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
        rejected_sinks: vec![sink.to_string()],
        unexpected_sinks: Vec::new(),
        sink_evidence: vec![DdlBarrierSinkEvidence {
            sink: sink.to_string(),
            status: "rejected".to_string(),
            ack_lsn: Some("0/16B9000".to_string()),
            schema_version: Some("schema-v2".to_string()),
            accepted: Some(true),
            detail: Some("accepted".to_string()),
            release_eligible: false,
            rejection_code: Some(code.to_string()),
            rejection_reason: Some(reason.to_string()),
        }],
        release_blockers: vec![format!("rejected or stale sink acknowledgements: {sink}")],
        release_blocker_codes: vec![code.to_string()],
        release_blocker_details: vec![ddl_release_blocker(
            "rejected_or_stale_ack",
            "rejected or stale sink acknowledgements",
            &[sink],
        )],
        release_actions: vec![ddl_release_action("replace_rejected_sink_ack", &[sink])],
        release_gates: vec![ddl_release_gate(
            "post_ddl_dml_release",
            false,
            "release_dml is false",
        )],
        release_dml: false,
        requires_global_partition_pause: false,
    }
}
