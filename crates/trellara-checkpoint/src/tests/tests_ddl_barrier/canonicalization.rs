use super::*;

#[tokio::test]
async fn ddl_barrier_store_canonicalizes_barrier_and_ack_lsns() {
    let store = InMemoryCheckpointStore::new();
    let flow = ddl_flow();
    store
        .record_ddl_barrier(DdlBarrier {
            source_id: flow.source_id.clone(),
        database_id: "retail".to_string(),
            dataset_id: flow.dataset_id.clone(),
            barrier_id: "ddl-barrier-canonical-lsn".to_string(),
            barrier_lsn: "00000000/016B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
            required_sinks: vec!["target_postgres".to_string()],
            requires_global_partition_pause: false,
        })
        .await
        .expect("record barrier");
    store
        .record_ddl_barrier_ack(DdlBarrierAck {
            source_id: flow.source_id.clone(),
            database_id: "retail".to_string(),
            dataset_id: flow.dataset_id.clone(),
            barrier_id: "ddl-barrier-canonical-lsn".to_string(),
            sink: "target_postgres".to_string(),
            ack_lsn: "00000000/016B9000".to_string(),
            schema_version: "schema-v2".to_string(),
            accepted: true,
            detail: "accepted target DDL".to_string(),
        })
        .await
        .expect("record ack");

    let summary = store
        .ddl_barrier_summary(&flow, "ddl-barrier-canonical-lsn")
        .await
        .expect("summary")
        .expect("barrier summary");

    assert_eq!(summary.barrier_lsn, "0/16B8000");
    assert!(summary.sink_evidence.iter().any(|evidence| {
        evidence.sink == "target_postgres" && evidence.ack_lsn.as_deref() == Some("0/16B9000")
    }));
}
