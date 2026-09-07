use super::*;

#[tokio::test]
async fn ddl_barrier_summary_rejects_stale_or_mismatched_acks() {
    let store = InMemoryCheckpointStore::new();
    let flow = ddl_flow();
    store
        .record_ddl_barrier(DdlBarrier {
            source_id: flow.source_id.clone(),
        database_id: "retail".to_string(),
            dataset_id: flow.dataset_id.clone(),
            barrier_id: "ddl-barrier-reject".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
            required_sinks: vec!["target_postgres".to_string(), "raw_cdc_lake".to_string()],
            requires_global_partition_pause: false,
        })
        .await
        .expect("record barrier");
    store
        .record_ddl_barrier_ack(DdlBarrierAck {
            source_id: flow.source_id.clone(),
            database_id: "retail".to_string(),
            dataset_id: flow.dataset_id.clone(),
            barrier_id: "ddl-barrier-reject".to_string(),
            sink: "target_postgres".to_string(),
            ack_lsn: "0/16B7000".to_string(),
            schema_version: "schema-v2".to_string(),
            accepted: true,
            detail: "stale ack".to_string(),
        })
        .await
        .expect("record stale ack");
    store
        .record_ddl_barrier_ack(DdlBarrierAck {
            source_id: flow.source_id.clone(),
            database_id: "retail".to_string(),
            dataset_id: flow.dataset_id.clone(),
            barrier_id: "ddl-barrier-reject".to_string(),
            sink: "raw_cdc_lake".to_string(),
            ack_lsn: "0/16B9000".to_string(),
            schema_version: "schema-v1".to_string(),
            accepted: true,
            detail: "wrong schema".to_string(),
        })
        .await
        .expect("record mismatched ack");

    let summary = store
        .ddl_barrier_summary(&flow, "ddl-barrier-reject")
        .await
        .expect("load summary")
        .expect("summary");

    assert!(!summary.release_dml);
    assert!(summary.pending_sinks.is_empty());
    assert_eq!(
        summary.rejected_sinks,
        vec!["raw_cdc_lake", "target_postgres"]
    );
    assert_eq!(
        summary.release_blockers,
        vec!["rejected or stale sink acknowledgements: raw_cdc_lake, target_postgres"]
    );
    assert_eq!(
        summary.release_blocker_codes,
        vec!["ack_lsn_before_barrier", "schema_version_mismatch"]
    );
    assert_eq!(summary.release_blocker_details.len(), 1);
    assert_eq!(
        summary.release_blocker_details[0].code,
        "rejected_or_stale_ack"
    );
    assert_eq!(
        summary.release_blocker_details[0].message,
        "rejected or stale sink acknowledgements"
    );
    assert_eq!(
        summary.release_blocker_details[0].sinks,
        vec!["raw_cdc_lake", "target_postgres"]
    );
    assert_eq!(
        summary.release_blocker_details[0].evidence,
        "barrier_id=ddl-barrier-reject barrier_lsn=0/16B8000 schema_version=schema-v2 rejected_sinks=raw_cdc_lake, target_postgres rejection_codes=raw_cdc_lake:schema_version_mismatch, target_postgres:ack_lsn_before_barrier"
    );
    assert_eq!(summary.release_actions.len(), 1);
    assert_eq!(summary.release_actions[0].code, "replace_rejected_sink_ack");
    assert_eq!(
        summary.release_actions[0].sinks,
        vec!["raw_cdc_lake", "target_postgres"]
    );
    let target_postgres = summary
        .sink_evidence
        .iter()
        .find(|evidence| evidence.sink == "target_postgres")
        .expect("target evidence");
    assert_eq!(
        target_postgres.rejection_reason.as_deref(),
        Some("ack_lsn 0/16B7000 is before barrier_lsn")
    );
    assert_eq!(
        target_postgres.rejection_code.as_deref(),
        Some("ack_lsn_before_barrier")
    );
    let raw_cdc_lake = summary
        .sink_evidence
        .iter()
        .find(|evidence| evidence.sink == "raw_cdc_lake")
        .expect("lake evidence");
    assert_eq!(
        raw_cdc_lake.rejection_reason.as_deref(),
        Some("schema_version schema-v1 does not match required schema-v2")
    );
    assert_eq!(
        raw_cdc_lake.rejection_code.as_deref(),
        Some("schema_version_mismatch")
    );
}
