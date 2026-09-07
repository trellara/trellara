use super::*;

pub(super) fn request(watermarks: PartitionWatermarkSummary) -> PartitionVisibilityDdlAckRequest {
    PartitionVisibilityDdlAckRequest {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "dataset-a".to_string(),
        barrier_id: "ddl-barrier-partitioned".to_string(),
        barrier_lsn: "0/16B9000".to_string(),
        schema_version: "schema-v2".to_string(),
        watermarks,
    }
}

pub(super) fn complete_watermarks() -> PartitionWatermarkSummary {
    PartitionWatermarkSummary::from_checkpoints(
        "source-a",
        "dataset-a",
        2,
        vec![partition(0, "0/16BA000"), partition(1, "0/16B9000")],
    )
    .expect("partition watermarks")
}

pub(super) fn partition(partition_id: u32, last_applied_lsn: &str) -> PartitionCheckpoint {
    PartitionCheckpoint {
        source_id: "source-a".to_string(),
        dataset_id: "dataset-a".to_string(),
        partition_id,
        last_durable_lsn: last_applied_lsn.to_string(),
        last_applied_lsn: last_applied_lsn.to_string(),
    }
}

pub(super) fn partitioned_barrier() -> DdlBarrier {
    DdlBarrier {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "dataset-a".to_string(),
        barrier_id: "ddl-barrier-partitioned".to_string(),
        barrier_lsn: "0/16B9000".to_string(),
        schema_version: "schema-v2".to_string(),
        cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
        required_sinks: vec![
            "target_postgres".to_string(),
            "raw_cdc_lake".to_string(),
            "spark_derived_views".to_string(),
            "partition_visibility".to_string(),
        ],
        requires_global_partition_pause: true,
    }
}

pub(super) fn target_ack() -> DdlBarrierAck {
    ack("target_postgres")
}

pub(super) fn raw_lake_ack() -> DdlBarrierAck {
    ack("raw_cdc_lake")
}

pub(super) fn spark_ack() -> DdlBarrierAck {
    ack("spark_derived_views")
}

fn ack(sink: &str) -> DdlBarrierAck {
    let detail = match sink {
        "target_postgres" => {
            target_postgres_ddl_ack_detail_with_digests(1, &"a".repeat(64), &["b".repeat(64)])
        }
        "raw_cdc_lake" => format!(
            "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest={}; release_gate=post_ddl_dml_release",
            "c".repeat(64)
        ),
        "spark_derived_views" => format!(
            "Spark-derived views accepted 2 regenerated templates; template_digest={}; accepted_by=platform-review; release_gate=post_ddl_dml_release",
            "d".repeat(64)
        ),
        _ => format!("{sink} accepted schema-v2"),
    };
    DdlBarrierAck {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "dataset-a".to_string(),
        barrier_id: "ddl-barrier-partitioned".to_string(),
        sink: sink.to_string(),
        ack_lsn: "0/16B9000".to_string(),
        schema_version: "schema-v2".to_string(),
        accepted: true,
        detail,
    }
}
