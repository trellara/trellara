use trellara_protocol::{PartitionChunk, TransactionCommitMarker, TransactionManifest};

use crate::barrier_header_context::HeaderContext;
use crate::barrier_header_payload::validate_header_payload_field;
use crate::barrier_manifest_evidence_validation::validate_manifest_evidence_header_context;
use crate::ApplyWorkerResult;

const PARTITION_PARALLEL_DML: &str = "partition_parallel_dml";
const PARTITION_PARALLEL_SAFE: &str = "true";
const DDL_BARRIER_NOT_REQUIRED: &str = "false";
const DML_REPLAY_AFTER_DDL_BARRIER_NOT_REQUIRED: &str = "false";
const PARTITION_PARALLEL_DML_REASON: &str =
    "DML-only transaction can be partitioned without a DDL barrier";

pub(crate) fn validate_chunk_header_context(
    context: &HeaderContext,
    chunk: &PartitionChunk,
) -> ApplyWorkerResult<()> {
    validate_partitioned_scale_decision(context)?;
    validate_header_payload_field(
        "transaction_id",
        &context.transaction_id,
        &chunk.transaction_id,
    )?;
    validate_header_payload_field(
        "partition_id",
        &context.partition_id,
        &chunk.partition_id.to_string(),
    )?;
    validate_header_payload_field(
        "partition_event_count",
        &context.partition_event_count,
        &chunk.changes.len().to_string(),
    )?;
    validate_header_payload_field(
        "partition_checksum",
        &context.partition_checksum,
        &chunk.checksum.to_string(),
    )
}

pub(crate) fn validate_manifest_header_context(
    context: &HeaderContext,
    manifest: &TransactionManifest,
) -> ApplyWorkerResult<()> {
    validate_partitioned_scale_decision(context)?;
    validate_header_payload_field(
        "transaction_id",
        &context.transaction_id,
        &manifest.transaction_id,
    )?;
    validate_header_payload_field(
        "commit_lsn",
        &context.commit_lsn,
        &manifest.source_commit_lsn,
    )?;
    validate_header_payload_field(
        "global_event_count",
        &context.global_event_count,
        &manifest.global_event_count.to_string(),
    )?;
    validate_header_payload_field(
        "partition_count",
        &context.partition_count,
        &manifest.partitions.len().to_string(),
    )?;
    validate_manifest_evidence_header_context(context, manifest)
}

pub(crate) fn validate_marker_header_context(
    context: &HeaderContext,
    marker: &TransactionCommitMarker,
) -> ApplyWorkerResult<()> {
    validate_partitioned_scale_decision(context)?;
    validate_header_payload_field(
        "transaction_id",
        &context.transaction_id,
        &marker.transaction_id,
    )?;
    validate_header_payload_field("commit_lsn", &context.commit_lsn, &marker.source_commit_lsn)?;
    validate_header_payload_field(
        "global_event_count",
        &context.global_event_count,
        &marker.global_event_count.to_string(),
    )?;
    validate_header_payload_field(
        "partition_count",
        &context.partition_count,
        &marker.participating_partition_count.to_string(),
    )?;
    validate_header_payload_field(
        "manifest_checksum",
        &context.manifest_checksum,
        &marker.manifest_checksum.to_string(),
    )
}

fn validate_partitioned_scale_decision(context: &HeaderContext) -> ApplyWorkerResult<()> {
    validate_header_payload_field(
        "partitioned_scale_decision",
        &context.partitioned_scale_decision,
        PARTITION_PARALLEL_DML,
    )?;
    validate_header_payload_field(
        "partition_parallel_safe",
        &context.partition_parallel_safe,
        PARTITION_PARALLEL_SAFE,
    )?;
    validate_header_payload_field(
        "requires_ddl_barrier",
        &context.requires_ddl_barrier,
        DDL_BARRIER_NOT_REQUIRED,
    )?;
    validate_header_payload_field(
        "dml_replay_after_ddl_barrier_required",
        &context.dml_replay_after_ddl_barrier_required,
        DML_REPLAY_AFTER_DDL_BARRIER_NOT_REQUIRED,
    )?;
    validate_header_payload_field(
        "partitioned_scale_reason",
        &context.partitioned_scale_reason,
        PARTITION_PARALLEL_DML_REASON,
    )
}
