use trellara_checkpoint::{
    partition_visibility_ddl_ack_evidence, target_postgres_ddl_ack_detail_with_boundary,
    DdlBarrierAck, PartitionVisibilityDdlAckRequest, PartitionWatermarkSummary,
};
use trellara_lake::{
    raw_cdc_lake_ddl_ack_evidence, spark_derived_views_ddl_ack_evidence, RawCdcLakeDdlAckRequest,
    SparkDerivedViewsDdlAckRequest,
};

use crate::ddl_barrier_ack_input::{
    required, required_clean, validate_ack_lsn, validate_barrier_id, validate_barrier_lsn,
    validate_detail, validate_schema_version, validate_sink,
};
use crate::ddl_barrier_ack_partition_input::partition_checkpoints;
use crate::{config_source_database_id, validate_sha256_digest, validate_statement_digests};
use crate::{DdlBarrierAckArgs, Result, TrellaraConfig};

pub(crate) fn ddl_barrier_ack_from_args(
    config: &TrellaraConfig,
    args: &DdlBarrierAckArgs,
) -> Result<DdlBarrierAck> {
    validate_barrier_id(&args.barrier_id)?;
    validate_sink(&args.sink)?;
    validate_ack_lsn(&args.ack_lsn)?;
    validate_schema_version(&args.schema_version)?;
    if !args.accepted {
        return generic_ack(config, args);
    }
    match args.sink.as_str() {
        "raw_cdc_lake" => raw_cdc_lake_ack(config, args),
        "spark_derived_views" => spark_derived_views_ack(config, args),
        "target_postgres" => target_postgres_ack(config, args),
        "partition_visibility" => partition_visibility_ack(config, args),
        _ => generic_ack(config, args),
    }
}

fn target_postgres_ack(config: &TrellaraConfig, args: &DdlBarrierAckArgs) -> Result<DdlBarrierAck> {
    let plan_sha256 = required_clean(args.plan_sha256.as_deref(), "plan_sha256")?;
    validate_sha256_digest("plan-sha256", plan_sha256)?;
    validate_statement_digests(&args.statement_sha256s)?;
    if let Some(barrier_lsn) = args.barrier_lsn.as_deref() {
        validate_barrier_lsn(barrier_lsn)?;
    }
    Ok(DdlBarrierAck {
        source_id: config.source.id.clone(),
        database_id: config_source_database_id(config),
        dataset_id: config.dataset.id.clone(),
        barrier_id: args.barrier_id.clone(),
        sink: args.sink.clone(),
        ack_lsn: args.ack_lsn.clone(),
        schema_version: args.schema_version.clone(),
        accepted: true,
        detail: target_postgres_ddl_ack_detail_with_boundary(
            args.statement_sha256s.len(),
            plan_sha256,
            &args.statement_sha256s,
            args.barrier_lsn.as_deref(),
        ),
    })
}

fn raw_cdc_lake_ack(config: &TrellaraConfig, args: &DdlBarrierAckArgs) -> Result<DdlBarrierAck> {
    Ok(raw_cdc_lake_ddl_ack_evidence(RawCdcLakeDdlAckRequest {
        source_id: config.source.id.clone(),
        database_id: config_source_database_id(config),
        dataset_id: config.dataset.id.clone(),
        barrier_id: args.barrier_id.clone(),
        ack_lsn: args.ack_lsn.clone(),
        schema_version: args.schema_version.clone(),
        epoch_id: required_clean(args.epoch_id.as_deref(), "epoch_id")?.to_string(),
        metadata_table: required_clean(args.metadata_table.as_deref(), "metadata_table")?
            .to_string(),
        partition_metadata_table: required_clean(
            args.partition_metadata_table.as_deref(),
            "partition_metadata_table",
        )?
        .to_string(),
        manifest_digest: required_clean(args.manifest_digest.as_deref(), "manifest_digest")?
            .to_string(),
    })?
    .into_barrier_ack())
}

fn spark_derived_views_ack(
    config: &TrellaraConfig,
    args: &DdlBarrierAckArgs,
) -> Result<DdlBarrierAck> {
    Ok(
        spark_derived_views_ddl_ack_evidence(SparkDerivedViewsDdlAckRequest {
            source_id: config.source.id.clone(),
            database_id: config_source_database_id(config),
            dataset_id: config.dataset.id.clone(),
            barrier_id: args.barrier_id.clone(),
            ack_lsn: args.ack_lsn.clone(),
            schema_version: args.schema_version.clone(),
            template_digest: required_clean(args.template_digest.as_deref(), "template_digest")?
                .to_string(),
            accepted_by: required_clean(args.accepted_by.as_deref(), "accepted_by")?.to_string(),
            view_count: required(args.view_count, "view_count")?,
        })?
        .into_barrier_ack(),
    )
}

fn partition_visibility_ack(
    config: &TrellaraConfig,
    args: &DdlBarrierAckArgs,
) -> Result<DdlBarrierAck> {
    let watermarks = PartitionWatermarkSummary::from_checkpoints(
        &config.source.id,
        &config.dataset.id,
        required(args.expected_partition_count, "expected_partition_count")?,
        partition_checkpoints(
            config,
            &args.partition_durable_lsns,
            &args.partition_applied_lsns,
        )?,
    )?;
    Ok(
        partition_visibility_ddl_ack_evidence(PartitionVisibilityDdlAckRequest {
            source_id: config.source.id.clone(),
            database_id: config_source_database_id(config),
            dataset_id: config.dataset.id.clone(),
            barrier_id: args.barrier_id.clone(),
            barrier_lsn: required_clean(args.barrier_lsn.as_deref(), "barrier_lsn")?.to_string(),
            schema_version: args.schema_version.clone(),
            watermarks,
        })?
        .into_barrier_ack(),
    )
}

fn generic_ack(config: &TrellaraConfig, args: &DdlBarrierAckArgs) -> Result<DdlBarrierAck> {
    validate_detail(&args.detail)?;
    Ok(DdlBarrierAck {
        source_id: config.source.id.clone(),
        database_id: config_source_database_id(config),
        dataset_id: config.dataset.id.clone(),
        barrier_id: args.barrier_id.clone(),
        sink: args.sink.clone(),
        ack_lsn: args.ack_lsn.clone(),
        schema_version: args.schema_version.clone(),
        accepted: args.accepted,
        detail: args.detail.clone(),
    })
}
