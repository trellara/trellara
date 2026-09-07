use super::super::*;

#[tokio::test]
async fn target_ddl_ack_evidence_records_into_barrier_store() {
    let store = InMemoryCheckpointStore::new();
    let flow = DdlBarrierLookup::new("source-a", "retail", "dataset-a");
    store
        .record_ddl_barrier(DdlBarrier {
            source_id: flow.source_id.clone(),
        database_id: "retail".to_string(),
            dataset_id: flow.dataset_id.clone(),
            barrier_id: "ddl-barrier-123".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
            required_sinks: vec!["target_postgres".to_string(), "raw_cdc_lake".to_string()],
            requires_global_partition_pause: false,
        })
        .await
        .expect("record barrier");

    valid_outcome()
        .target_ack_evidence_for_database(
            "source-a",
            "retail",
            "dataset-a",
            "0/16B9000",
            "schema-v2",
        )
        .expect("ack evidence")
        .record_barrier_ack(&store)
        .await
        .expect("record target ack");

    let summary = store
        .ddl_barrier_summary(&flow, "ddl-barrier-123")
        .await
        .expect("summary")
        .expect("barrier summary");

    assert_eq!(summary.acked_sinks, vec!["target_postgres"]);
    assert_eq!(summary.pending_sinks, vec!["raw_cdc_lake"]);
    assert!(!summary.release_dml);
    assert!(summary.sink_evidence.iter().any(|evidence| {
        evidence.sink == "target_postgres"
            && evidence.release_eligible
            && evidence.detail.as_deref().is_some_and(|detail| {
                detail.contains("target Postgres applied 1 DDL statements")
                    && detail.contains("post_ddl_dml_release")
            })
    }));
}
