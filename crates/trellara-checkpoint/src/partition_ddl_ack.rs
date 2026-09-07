use crate::partition_ddl_ack_validation::validate_request;
use crate::partition_visibility_ddl_ack_detail;
use crate::{
    partition_watermark_summary_sha256, CheckpointError, DdlBarrierAck, DdlBarrierStore,
    PartitionWatermarkSummary, Result,
};

const PARTITION_VISIBILITY_SINK: &str = "partition_visibility";
const RELEASE_GATE: &str = "post_ddl_dml_release";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartitionVisibilityDdlAckRequest {
    pub source_id: String,
    pub database_id: String,
    pub dataset_id: String,
    pub barrier_id: String,
    pub barrier_lsn: String,
    pub schema_version: String,
    pub watermarks: PartitionWatermarkSummary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartitionVisibilityDdlAckEvidence {
    pub source_id: String,
    pub database_id: String,
    pub dataset_id: String,
    pub barrier_id: String,
    pub sink: String,
    pub ack_lsn: String,
    pub durable_lsn: String,
    pub schema_version: String,
    pub expected_partition_count: u32,
    pub observed_partition_count: u32,
    pub partition_watermark_sha256: String,
    pub release_gate: String,
}

pub fn partition_visibility_ddl_ack_evidence(
    request: PartitionVisibilityDdlAckRequest,
) -> Result<PartitionVisibilityDdlAckEvidence> {
    validate_request(&request)?;
    let ack_lsn = request
        .watermarks
        .global_applied_lsn
        .clone()
        .ok_or_else(|| store_error("partition visibility DDL ack is missing global_applied_lsn"))?;
    let durable_lsn = request
        .watermarks
        .global_durable_lsn
        .clone()
        .ok_or_else(|| store_error("partition visibility DDL ack is missing global_durable_lsn"))?;
    Ok(PartitionVisibilityDdlAckEvidence {
        source_id: request.source_id,
        database_id: request.database_id,
        dataset_id: request.dataset_id,
        barrier_id: request.barrier_id,
        sink: PARTITION_VISIBILITY_SINK.to_string(),
        ack_lsn,
        durable_lsn,
        schema_version: request.schema_version,
        expected_partition_count: request.watermarks.expected_partition_count,
        observed_partition_count: request.watermarks.observed_partition_count,
        partition_watermark_sha256: partition_watermark_summary_sha256(&request.watermarks),
        release_gate: RELEASE_GATE.to_string(),
    })
}

impl PartitionVisibilityDdlAckEvidence {
    pub async fn record_barrier_ack<S: DdlBarrierStore + ?Sized>(self, store: &S) -> Result<()> {
        store.record_ddl_barrier_ack(self.into_barrier_ack()).await
    }

    pub fn into_barrier_ack(self) -> DdlBarrierAck {
        let detail = partition_visibility_ddl_ack_detail(
            self.observed_partition_count,
            self.expected_partition_count,
            &self.durable_lsn,
            &self.ack_lsn,
            &self.partition_watermark_sha256,
        );
        DdlBarrierAck {
            source_id: self.source_id,
            database_id: self.database_id,
            dataset_id: self.dataset_id,
            barrier_id: self.barrier_id,
            sink: self.sink,
            ack_lsn: self.ack_lsn,
            schema_version: self.schema_version,
            accepted: true,
            detail,
        }
    }
}

fn store_error(message: impl Into<String>) -> CheckpointError {
    CheckpointError::Store(message.into())
}
