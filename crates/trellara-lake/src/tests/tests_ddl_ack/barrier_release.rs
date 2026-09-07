use super::*;
use trellara_checkpoint::{DdlBarrierLookup, InMemoryCheckpointStore};

#[tokio::test]
async fn raw_cdc_lake_ddl_ack_can_release_shared_barrier() {
    let store = InMemoryCheckpointStore::default();
    let flow = DdlBarrierLookup {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "dataset-a".to_string(),
    };
    store.record_ddl_barrier(shared_barrier()).await.unwrap();
    store.record_ddl_barrier_ack(target_ack()).await.unwrap();
    valid_ack().record_barrier_ack(&store).await.unwrap();

    let summary = store
        .ddl_barrier_summary(&flow, "ddl-barrier-123")
        .await
        .unwrap()
        .expect("barrier summary");

    assert!(summary.release_dml);
    assert!(summary.pending_sinks.is_empty());
    assert!(summary.sink_evidence.iter().any(|evidence| {
        evidence.sink == "raw_cdc_lake"
            && evidence.accepted == Some(true)
            && evidence
                .detail
                .as_deref()
                .is_some_and(|detail| detail.contains("epoch-2026-08-16T06"))
    }));
}
