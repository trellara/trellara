use trellara_checkpoint::{
    partition_visibility_ddl_ack_evidence, target_postgres_ddl_ack_detail_with_boundary,
    DdlBarrierAck, PartitionCheckpoint, PartitionVisibilityDdlAckRequest,
    PartitionWatermarkSummary,
};
use trellara_lake::{
    raw_cdc_lake_ddl_ack_evidence, spark_derived_views_ddl_ack_evidence, RawCdcLakeDdlAckRequest,
    SparkDerivedViewsDdlAckRequest,
};

use crate::{
    config_source_database_id,
    pilot_package_ddl_release_ack_samples::{
        raw_cdc_metadata_table, raw_cdc_partition_metadata_table, SAMPLE_ACK_LSN,
        SAMPLE_LAKE_EPOCH_ID, SAMPLE_LAKE_MANIFEST_DIGEST, SAMPLE_SPARK_ACCEPTED_BY,
        SAMPLE_SPARK_TEMPLATE_DIGEST, SAMPLE_SPARK_VIEW_COUNT,
    },
    DdlApplyPlanSummary, DdlPlanSummary, Result, TrellaraConfig,
};

pub(crate) fn target_postgres_ack(
    config: &TrellaraConfig,
    plan: &DdlPlanSummary,
    apply_plan: &DdlApplyPlanSummary,
    barrier_lsn: &str,
    schema_version: &str,
) -> Result<DdlBarrierAck> {
    let statement_sha256s = apply_plan
        .statements
        .iter()
        .map(|statement| statement.statement_sha256.clone())
        .collect::<Vec<_>>();

    Ok(DdlBarrierAck {
        source_id: config.source.id.clone(),
        database_id: config_source_database_id(config),
        dataset_id: config.dataset.id.clone(),
        barrier_id: plan.propagation.barrier_id.clone(),
        sink: "target_postgres".to_string(),
        ack_lsn: SAMPLE_ACK_LSN.to_string(),
        schema_version: schema_version.to_string(),
        accepted: true,
        detail: target_postgres_ddl_ack_detail_with_boundary(
            apply_plan.statement_count,
            &apply_plan.plan_sha256,
            &statement_sha256s,
            Some(barrier_lsn),
        ),
    })
}

pub(crate) fn raw_cdc_lake_ack(
    config: &TrellaraConfig,
    plan: &DdlPlanSummary,
    schema_version: &str,
) -> Result<DdlBarrierAck> {
    Ok(raw_cdc_lake_ddl_ack_evidence(RawCdcLakeDdlAckRequest {
        source_id: config.source.id.clone(),
        database_id: config_source_database_id(config),
        dataset_id: config.dataset.id.clone(),
        barrier_id: plan.propagation.barrier_id.clone(),
        ack_lsn: SAMPLE_ACK_LSN.to_string(),
        schema_version: schema_version.to_string(),
        epoch_id: SAMPLE_LAKE_EPOCH_ID.to_string(),
        metadata_table: raw_cdc_metadata_table(config),
        partition_metadata_table: raw_cdc_partition_metadata_table(config),
        manifest_digest: SAMPLE_LAKE_MANIFEST_DIGEST.to_string(),
    })?
    .into_barrier_ack())
}

pub(crate) fn spark_derived_views_ack(
    config: &TrellaraConfig,
    plan: &DdlPlanSummary,
    schema_version: &str,
) -> Result<DdlBarrierAck> {
    Ok(
        spark_derived_views_ddl_ack_evidence(SparkDerivedViewsDdlAckRequest {
            source_id: config.source.id.clone(),
            database_id: config_source_database_id(config),
            dataset_id: config.dataset.id.clone(),
            barrier_id: plan.propagation.barrier_id.clone(),
            ack_lsn: SAMPLE_ACK_LSN.to_string(),
            schema_version: schema_version.to_string(),
            template_digest: SAMPLE_SPARK_TEMPLATE_DIGEST.to_string(),
            accepted_by: SAMPLE_SPARK_ACCEPTED_BY.to_string(),
            view_count: SAMPLE_SPARK_VIEW_COUNT,
        })?
        .into_barrier_ack(),
    )
}

pub(crate) fn partition_visibility_ack(
    config: &TrellaraConfig,
    plan: &DdlPlanSummary,
    barrier_lsn: &str,
    schema_version: &str,
) -> Result<DdlBarrierAck> {
    let partition_count = config
        .dataset
        .partition
        .as_ref()
        .map(|partition| partition.partition_count)
        .unwrap_or(1);
    let checkpoints = (0..partition_count)
        .map(|partition_id| PartitionCheckpoint {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            partition_id,
            last_durable_lsn: SAMPLE_ACK_LSN.to_string(),
            last_applied_lsn: SAMPLE_ACK_LSN.to_string(),
        })
        .collect::<Vec<_>>();
    let watermarks = PartitionWatermarkSummary::from_checkpoints(
        &config.source.id,
        &config.dataset.id,
        partition_count,
        checkpoints,
    )?;

    Ok(
        partition_visibility_ddl_ack_evidence(PartitionVisibilityDdlAckRequest {
            source_id: config.source.id.clone(),
            database_id: config_source_database_id(config),
            dataset_id: config.dataset.id.clone(),
            barrier_id: plan.propagation.barrier_id.clone(),
            barrier_lsn: barrier_lsn.to_string(),
            schema_version: schema_version.to_string(),
            watermarks,
        })?
        .into_barrier_ack(),
    )
}
