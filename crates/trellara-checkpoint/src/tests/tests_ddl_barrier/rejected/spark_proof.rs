use super::*;

const DIGEST: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

#[test]
fn ddl_barrier_summary_rejects_spark_ack_without_template_evidence() {
    let summary = spark_ack_summary("accepted");

    assert_spark_evidence_is_insufficient(&summary);
}

#[test]
fn ddl_barrier_summary_rejects_spark_ack_without_reviewer() {
    let summary = spark_ack_summary(&format!(
        "Spark-derived views accepted 2 regenerated templates; template_digest={DIGEST}; release_gate=post_ddl_dml_release"
    ));

    assert_spark_evidence_is_insufficient(&summary);
}

#[test]
fn ddl_barrier_summary_rejects_spark_ack_with_spoofed_release_gate_detail() {
    let summary = spark_ack_summary(&format!(
        "Spark-derived views accepted 2 regenerated templates; template_digest={DIGEST}; accepted_by=platform-review; previous_release_gate=post_ddl_dml_release"
    ));

    assert_spark_evidence_is_insufficient(&summary);
}

#[test]
fn ddl_barrier_summary_accepts_spark_ack_with_template_review_evidence() {
    let summary = spark_ack_summary(&format!(
        "Spark-derived views accepted 2 regenerated templates; template_digest={DIGEST}; accepted_by=platform-review; release_gate=post_ddl_dml_release"
    ));

    assert!(summary.release_dml);
    assert!(summary.rejected_sinks.is_empty());
    assert!(summary.release_blocker_codes.is_empty());
    let evidence = summary
        .sink_evidence
        .iter()
        .find(|evidence| evidence.sink == "spark_derived_views")
        .expect("Spark evidence");
    assert!(evidence.release_eligible);
    assert_eq!(evidence.status, "acked");
}

fn spark_ack_summary(detail: &str) -> DdlBarrierSummary {
    DdlBarrierSummary::from_barrier_and_acks(
        DdlBarrier {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-spark-proof".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
            required_sinks: vec!["spark_derived_views".to_string()],
            requires_global_partition_pause: false,
        },
        vec![DdlBarrierAck {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-spark-proof".to_string(),
            sink: "spark_derived_views".to_string(),
            ack_lsn: "0/16B9000".to_string(),
            schema_version: "schema-v2".to_string(),
            accepted: true,
            detail: detail.to_string(),
        }],
    )
}

fn assert_spark_evidence_is_insufficient(summary: &DdlBarrierSummary) {
    assert!(!summary.release_dml);
    assert_eq!(summary.rejected_sinks, vec!["spark_derived_views"]);
    assert_eq!(
        summary.release_blocker_codes,
        vec!["insufficient_spark_derived_views_evidence"]
    );
    let evidence = summary
        .sink_evidence
        .iter()
        .find(|evidence| evidence.sink == "spark_derived_views")
        .expect("Spark evidence");
    assert_eq!(
        evidence.rejection_code.as_deref(),
        Some("insufficient_spark_derived_views_evidence")
    );
    assert!(evidence
        .rejection_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("template_digest")));
    assert!(!evidence.release_eligible);
}
