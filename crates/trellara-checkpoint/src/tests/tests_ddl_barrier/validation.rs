use super::*;

mod ack;
mod cdc_boundary;
mod summary_constructor;

#[tokio::test]
async fn ddl_barrier_rejects_partition_pause_without_partition_visibility_sink() {
    let store = InMemoryCheckpointStore::new();
    let flow = ddl_flow();
    let error = store
        .record_ddl_barrier(DdlBarrier {
            source_id: flow.source_id,
        database_id: "retail".to_string(),
            dataset_id: flow.dataset_id,
            barrier_id: "ddl-barrier-missing-partition-visibility".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
            required_sinks: vec!["target_postgres".to_string(), "raw_cdc_lake".to_string()],
            requires_global_partition_pause: true,
        })
        .await
        .expect_err("partition pause must require partition visibility sink");

    assert!(error.to_string().contains(
        "requires partition_visibility acknowledgement when global partition pause is enabled"
    ));
}

#[tokio::test]
async fn ddl_barrier_rejects_invalid_barrier_lsn() {
    for barrier_lsn in [
        "not-a-lsn",
        "0/0",
        "xyz/16B9000",
        "100000000/0",
        "0/100000000",
    ] {
        let error = InMemoryCheckpointStore::new()
            .record_ddl_barrier(DdlBarrier {
                source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
                dataset_id: "sales".to_string(),
                barrier_id: format!("ddl-barrier-{barrier_lsn}"),
                barrier_lsn: barrier_lsn.to_string(),
                schema_version: "schema-v2".to_string(),
                cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
                required_sinks: vec!["target_postgres".to_string()],
                requires_global_partition_pause: false,
            })
            .await
            .expect_err("invalid barrier lsn");

        assert!(error.to_string().contains("barrier_lsn"));
        assert!(error.to_string().contains("non-zero PostgreSQL LSN"));
    }
}

#[tokio::test]
async fn ddl_barrier_rejects_invalid_ack_lsn() {
    for ack_lsn in [
        "not-a-lsn",
        "0/0",
        "xyz/16B9000",
        "100000000/0",
        "0/100000000",
    ] {
        let error = InMemoryCheckpointStore::new()
            .record_ddl_barrier_ack(DdlBarrierAck {
                source_id: "source-a".to_string(),
                database_id: "retail".to_string(),
                dataset_id: "sales".to_string(),
                barrier_id: "ddl-barrier-invalid-ack".to_string(),
                sink: "target_postgres".to_string(),
                ack_lsn: ack_lsn.to_string(),
                schema_version: "schema-v2".to_string(),
                accepted: true,
                detail: "accepted".to_string(),
            })
            .await
            .expect_err("invalid ack lsn");

        assert!(error.to_string().contains("ack_lsn"));
        assert!(error.to_string().contains("non-zero PostgreSQL LSN"));
    }
}

#[tokio::test]
async fn ddl_barrier_rejects_required_sinks_with_surrounding_whitespace() {
    let error = InMemoryCheckpointStore::new()
        .record_ddl_barrier(DdlBarrier {
            source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-spaced-sink".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
            required_sinks: vec!["target_postgres".to_string(), " raw_cdc_lake ".to_string()],
            requires_global_partition_pause: false,
        })
        .await
        .expect_err("spaced required sink");

    assert!(error
        .to_string()
        .contains("required sink names must not contain surrounding whitespace"));
}

#[tokio::test]
async fn ddl_barrier_rejects_schema_version_with_surrounding_whitespace() {
    let error = InMemoryCheckpointStore::new()
        .record_ddl_barrier(DdlBarrier {
            source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-spaced-schema".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: " schema-v2 ".to_string(),
            cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
            required_sinks: vec!["target_postgres".to_string()],
            requires_global_partition_pause: false,
        })
        .await
        .expect_err("spaced barrier schema version");

    assert!(error
        .to_string()
        .contains("schema_version must not contain surrounding whitespace"));
}

#[tokio::test]
async fn ddl_barrier_rejects_identity_fields_with_surrounding_whitespace() {
    for (source_id, dataset_id, barrier_id, field) in [
        (
            " source-a ",
            "sales",
            "ddl-barrier-spaced-source",
            "source_id",
        ),
        (
            "source-a",
            " sales ",
            "ddl-barrier-spaced-dataset",
            "dataset_id",
        ),
        ("source-a", "sales", " ddl-barrier-spaced-id ", "id"),
    ] {
        let error = InMemoryCheckpointStore::new()
            .record_ddl_barrier(DdlBarrier {
                source_id: source_id.to_string(),
        database_id: "retail".to_string(),
                dataset_id: dataset_id.to_string(),
                barrier_id: barrier_id.to_string(),
                barrier_lsn: "0/16B8000".to_string(),
                schema_version: "schema-v2".to_string(),
                cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
                required_sinks: vec!["target_postgres".to_string()],
                requires_global_partition_pause: false,
            })
            .await
            .expect_err("spaced barrier identity");

        assert!(error.to_string().contains(field));
        assert!(error
            .to_string()
            .contains("must not contain surrounding whitespace"));
    }
}

#[tokio::test]
async fn ddl_barrier_summary_rejects_lookup_identity_with_surrounding_whitespace() {
    for (flow, barrier_id, field) in [
        (
            DdlBarrierLookup::new(" source-a ", "retail", "sales"),
            "ddl-barrier-lookup",
            "source_id",
        ),
        (
            DdlBarrierLookup::new("source-a", "retail", " sales "),
            "ddl-barrier-lookup",
            "dataset_id",
        ),
        (
            DdlBarrierLookup::new("source-a", "retail", "sales"),
            " ddl-barrier-lookup ",
            "barrier_id",
        ),
    ] {
        let error = InMemoryCheckpointStore::new()
            .ddl_barrier_summary(&flow, barrier_id)
            .await
            .expect_err("spaced lookup identity");

        assert!(error.to_string().contains(field));
        assert!(error
            .to_string()
            .contains("must not contain surrounding whitespace"));
    }
}
