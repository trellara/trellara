use super::*;
use trellara_checkpoint::{DdlBarrier, DdlBarrierAck};

pub(super) const MANIFEST_DIGEST: &str =
    "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

pub(super) fn valid_ack() -> RawCdcLakeDdlAckEvidence {
    raw_cdc_lake_ddl_ack_evidence(RawCdcLakeDdlAckRequest {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "dataset-a".to_string(),
        barrier_id: "ddl-barrier-123".to_string(),
        ack_lsn: "0/16B9000".to_string(),
        schema_version: "schema-v2".to_string(),
        epoch_id: "epoch-2026-08-16T06".to_string(),
        metadata_table: "_trellara_raw_cdc_epochs".to_string(),
        partition_metadata_table: "_trellara_epoch_partitions".to_string(),
        manifest_digest: MANIFEST_DIGEST.to_string(),
    })
    .expect("raw CDC lake DDL ack")
}

pub(super) fn shared_barrier() -> DdlBarrier {
    DdlBarrier {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "dataset-a".to_string(),
        barrier_id: "ddl-barrier-123".to_string(),
        barrier_lsn: "0/16B9000".to_string(),
        schema_version: "schema-v2".to_string(),
        cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
        required_sinks: vec!["target_postgres".to_string(), "raw_cdc_lake".to_string()],
        requires_global_partition_pause: false,
    }
}

pub(super) fn target_ack() -> DdlBarrierAck {
    DdlBarrierAck {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "dataset-a".to_string(),
        barrier_id: "ddl-barrier-123".to_string(),
        sink: "target_postgres".to_string(),
        ack_lsn: "0/16B9000".to_string(),
        schema_version: "schema-v2".to_string(),
        accepted: true,
        detail: trellara_checkpoint::target_postgres_ddl_ack_detail_with_digests(
            1,
            &"a".repeat(64),
            &["b".repeat(64)],
        ),
    }
}
