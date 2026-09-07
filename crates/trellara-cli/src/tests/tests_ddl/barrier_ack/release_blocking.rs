use super::*;

#[test]
fn rejected_typed_sink_ack_blocks_barrier_release() {
    let mut args = base_args("raw_cdc_lake");
    args.accepted = false;
    args.detail = "lake metadata did not include schema-v2".to_string();
    let ack = ddl_barrier_ack_from_args(&config(), &args).expect("ack");

    let summary = DdlBarrierSummary::try_from_barrier_and_acks(shared_barrier(), vec![ack])
        .expect("valid barrier summary");

    assert!(!summary.release_dml);
    assert_eq!(summary.rejected_sinks, vec!["raw_cdc_lake"]);
    assert!(summary.release_blockers.iter().any(|blocker| {
        blocker.contains("rejected or stale sink acknowledgements: raw_cdc_lake")
    }));
    assert!(summary.sink_evidence.iter().any(|evidence| {
        evidence.sink == "raw_cdc_lake"
            && evidence.status == "rejected"
            && evidence.accepted == Some(false)
            && evidence.rejection_reason.as_deref() == Some("sink rejected the schema barrier")
    }));
}
