use super::*;

#[test]
fn ddl_barrier_summary_blocks_duplicate_required_sink_acks() {
    let summary = duplicate_summary(vec![
        duplicate_ack("0/16B9000", "schema-v2", true, "accepted"),
        duplicate_ack("0/16BA000", "schema-v1", true, "wrong schema"),
    ]);

    assert!(!summary.release_dml);
    assert!(summary.pending_sinks.is_empty());
    assert_eq!(summary.rejected_sinks, vec!["target_postgres"]);
    assert_eq!(
        summary.release_blockers,
        vec!["rejected or stale sink acknowledgements: target_postgres"]
    );
    assert_eq!(
        summary.release_blocker_codes,
        vec!["duplicate_required_ack"]
    );
    assert_eq!(
        summary.release_blocker_details[0].code,
        "rejected_or_stale_ack"
    );
    let evidence = summary
        .sink_evidence
        .iter()
        .find(|evidence| evidence.sink == "target_postgres")
        .expect("target evidence");
    assert_eq!(evidence.status, "rejected");
    assert!(!evidence.release_eligible);
    assert_eq!(
        evidence.rejection_reason.as_deref(),
        Some("multiple acknowledgements for required sink must be reconciled")
    );
    assert_eq!(
        evidence.rejection_code.as_deref(),
        Some("duplicate_required_ack")
    );
    assert_eq!(
        evidence.detail.as_deref(),
        Some("2 acknowledgements recorded for sink target_postgres")
    );
}

#[test]
fn ddl_barrier_summary_duplicate_required_sink_acks_are_order_independent() {
    let first = duplicate_ack("0/16B9000", "schema-v2", true, "accepted");
    let second = duplicate_ack("0/16BA000", "schema-v1", true, "wrong schema");
    let forward = duplicate_summary(vec![first.clone(), second.clone()]);
    let reverse = duplicate_summary(vec![second, first]);

    assert_eq!(forward.release_dml, reverse.release_dml);
    assert_eq!(forward.rejected_sinks, reverse.rejected_sinks);
    assert_eq!(forward.release_blockers, reverse.release_blockers);
    assert_eq!(forward.sink_evidence, reverse.sink_evidence);
}

fn duplicate_summary(acks: Vec<DdlBarrierAck>) -> DdlBarrierSummary {
    DdlBarrierSummary::from_barrier_and_acks(
        DdlBarrier {
            source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-duplicate-required".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
            required_sinks: vec!["target_postgres".to_string()],
            requires_global_partition_pause: false,
        },
        acks,
    )
}
