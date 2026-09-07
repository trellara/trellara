use super::*;

const DIGEST: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

#[test]
fn ddl_barrier_summary_rejects_raw_cdc_lake_ack_without_epoch_evidence() {
    let summary = raw_cdc_lake_ack_summary("accepted");

    assert_raw_cdc_lake_evidence_is_insufficient(&summary);
}

#[test]
fn ddl_barrier_summary_rejects_raw_cdc_lake_ack_without_manifest_digest() {
    let summary = raw_cdc_lake_ack_summary(
        "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; release_gate=post_ddl_dml_release",
    );

    assert_raw_cdc_lake_evidence_is_insufficient(&summary);
}

#[test]
fn ddl_barrier_summary_rejects_raw_cdc_lake_ack_with_spoofed_release_gate_detail() {
    let summary = raw_cdc_lake_ack_summary(&format!(
        "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest={DIGEST}; previous_release_gate=post_ddl_dml_release"
    ));

    assert_raw_cdc_lake_evidence_is_insufficient(&summary);
}

#[test]
fn ddl_barrier_summary_accepts_raw_cdc_lake_ack_with_epoch_manifest_evidence() {
    let summary = raw_cdc_lake_ack_summary(&format!(
        "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest={DIGEST}; release_gate=post_ddl_dml_release"
    ));

    assert!(summary.release_dml);
    assert!(summary.rejected_sinks.is_empty());
    assert!(summary.release_blocker_codes.is_empty());
    let evidence = summary
        .sink_evidence
        .iter()
        .find(|evidence| evidence.sink == "raw_cdc_lake")
        .expect("raw CDC lake evidence");
    assert!(evidence.release_eligible);
    assert_eq!(evidence.status, "acked");
}

fn raw_cdc_lake_ack_summary(detail: &str) -> DdlBarrierSummary {
    DdlBarrierSummary::from_barrier_and_acks(
        DdlBarrier {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-lake-proof".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
            required_sinks: vec!["raw_cdc_lake".to_string()],
            requires_global_partition_pause: false,
        },
        vec![DdlBarrierAck {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-lake-proof".to_string(),
            sink: "raw_cdc_lake".to_string(),
            ack_lsn: "0/16B9000".to_string(),
            schema_version: "schema-v2".to_string(),
            accepted: true,
            detail: detail.to_string(),
        }],
    )
}

fn assert_raw_cdc_lake_evidence_is_insufficient(summary: &DdlBarrierSummary) {
    assert!(!summary.release_dml);
    assert_eq!(summary.rejected_sinks, vec!["raw_cdc_lake"]);
    assert_eq!(
        summary.release_blocker_codes,
        vec!["insufficient_raw_cdc_lake_evidence"]
    );
    let evidence = summary
        .sink_evidence
        .iter()
        .find(|evidence| evidence.sink == "raw_cdc_lake")
        .expect("raw CDC lake evidence");
    assert_eq!(
        evidence.rejection_code.as_deref(),
        Some("insufficient_raw_cdc_lake_evidence")
    );
    assert!(evidence
        .rejection_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("manifest_digest")));
    assert!(!evidence.release_eligible);
}
