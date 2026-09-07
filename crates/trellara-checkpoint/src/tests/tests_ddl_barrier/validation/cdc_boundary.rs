use super::*;

#[tokio::test]
async fn ddl_barrier_rejects_empty_cdc_transaction_boundary() {
    let error = InMemoryCheckpointStore::new()
        .record_ddl_barrier(DdlBarrier {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-missing-boundary".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: "".to_string(),
            required_sinks: vec!["target_postgres".to_string()],
            requires_global_partition_pause: false,
        })
        .await
        .expect_err("missing CDC transaction boundary");

    assert!(error.to_string().contains("cdc_transaction_boundary"));
}

#[tokio::test]
async fn ddl_barrier_rejects_non_canonical_cdc_boundary() {
    let error = InMemoryCheckpointStore::new()
        .record_ddl_barrier(DdlBarrier {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-wrong-boundary".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary:
                "target apply timestamp is the DDL barrier; post-DDL DML waits".to_string(),
            required_sinks: vec!["target_postgres".to_string()],
            requires_global_partition_pause: false,
        })
        .await
        .expect_err("non-canonical CDC boundary");

    assert!(error.to_string().contains("canonical source commit LSN"));
}

#[tokio::test]
async fn ddl_barrier_rejects_boundary_without_post_ddl_hold() {
    let error = InMemoryCheckpointStore::new()
        .record_ddl_barrier(DdlBarrier {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-no-hold".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: DDL_BARRIER_CDC_TRANSACTION_BOUNDARY.to_string(),
            required_sinks: vec!["target_postgres".to_string()],
            requires_global_partition_pause: false,
        })
        .await
        .expect_err("CDC boundary without post-DDL DML hold");

    assert!(error.to_string().contains("post-DDL DML is held"));
}

#[tokio::test]
async fn ddl_barrier_rejects_partial_propagation_policy_evidence() {
    let error = InMemoryCheckpointStore::new()
        .record_ddl_barrier(DdlBarrier {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-partial-policy".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: format!(
                "{DDL_BARRIER_CDC_TRANSACTION_BOUNDARY}; propagation_boundary={}; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
                trellara_protocol::DDL_PROPAGATION_CDC_BOUNDARY
            ),
            required_sinks: vec!["target_postgres".to_string()],
            requires_global_partition_pause: false,
        })
        .await
        .expect_err("partial propagation policy evidence");

    assert!(error
        .to_string()
        .contains("propagation_decisions must include"));
}

#[tokio::test]
async fn ddl_barrier_rejects_invalid_propagation_policy_digest() {
    let error = InMemoryCheckpointStore::new()
        .record_ddl_barrier(DdlBarrier {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-bad-policy-digest".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: format!(
                "{DDL_BARRIER_CDC_TRANSACTION_BOUNDARY}; propagation_boundary={}; propagation_decisions=auto_apply:1,manual_review:0,unsupported:0,target_ack_required:1; propagation_policy_sha256=short; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
                trellara_protocol::DDL_PROPAGATION_CDC_BOUNDARY
            ),
            required_sinks: vec!["target_postgres".to_string()],
            requires_global_partition_pause: false,
        })
        .await
        .expect_err("invalid propagation policy digest");

    assert!(error
        .to_string()
        .contains("propagation_policy_sha256 must be a 64-character"));
}
