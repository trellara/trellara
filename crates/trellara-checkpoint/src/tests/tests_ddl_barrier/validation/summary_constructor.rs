use super::*;

#[test]
fn ddl_barrier_summary_try_constructor_canonicalizes_lsns() {
    let summary = DdlBarrierSummary::try_from_barrier_and_acks(
        DdlBarrier {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-canonical-summary".to_string(),
            barrier_lsn: "0/016B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
            required_sinks: vec!["target_postgres".to_string()],
            requires_global_partition_pause: false,
        },
        vec![DdlBarrierAck {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-canonical-summary".to_string(),
            sink: "target_postgres".to_string(),
            ack_lsn: "0/016B9000".to_string(),
            schema_version: "schema-v2".to_string(),
            accepted: true,
            detail: target_postgres_ddl_ack_detail_with_digests(
                1,
                &"a".repeat(64),
                &["b".repeat(64)],
            ),
        }],
    )
    .expect("summary");

    assert_eq!(summary.barrier_lsn, "0/16B8000");
    assert_eq!(
        summary.sink_evidence[0].ack_lsn,
        Some("0/16B9000".to_string())
    );
    assert!(summary.release_dml);
}

#[test]
fn ddl_barrier_summary_try_constructor_rejects_ack_for_different_barrier() {
    let error = DdlBarrierSummary::try_from_barrier_and_acks(
        DdlBarrier {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-summary".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
            required_sinks: vec!["target_postgres".to_string()],
            requires_global_partition_pause: false,
        },
        vec![DdlBarrierAck {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-other".to_string(),
            sink: "target_postgres".to_string(),
            ack_lsn: "0/16B9000".to_string(),
            schema_version: "schema-v2".to_string(),
            accepted: true,
            detail: target_postgres_ddl_ack_detail_with_digests(
                1,
                &"a".repeat(64),
                &["b".repeat(64)],
            ),
        }],
    )
    .expect_err("mismatched ack rejected");

    assert!(error
        .to_string()
        .contains("belongs to a different barrier identity"));
}

#[test]
fn ddl_barrier_summary_try_constructor_rejects_ack_for_different_database() {
    let error = DdlBarrierSummary::try_from_barrier_and_acks(
        DdlBarrier {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-summary".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
            required_sinks: vec!["target_postgres".to_string()],
            requires_global_partition_pause: false,
        },
        vec![DdlBarrierAck {
            source_id: "source-a".to_string(),
            database_id: "analytics".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-summary".to_string(),
            sink: "target_postgres".to_string(),
            ack_lsn: "0/16B9000".to_string(),
            schema_version: "schema-v2".to_string(),
            accepted: true,
            detail: target_postgres_ddl_ack_detail_with_digests(
                1,
                &"a".repeat(64),
                &["b".repeat(64)],
            ),
        }],
    )
    .expect_err("database-mismatched ack rejected");

    assert!(error
        .to_string()
        .contains("belongs to a different barrier identity"));
}
