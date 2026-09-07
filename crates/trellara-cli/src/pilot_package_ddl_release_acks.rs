use trellara_checkpoint::DdlBarrierAck;

use crate::{
    config_source_database_id,
    pilot_package_ddl_release_ack_samples::SAMPLE_ACK_LSN,
    pilot_package_ddl_release_sink_acks::{
        partition_visibility_ack, raw_cdc_lake_ack, spark_derived_views_ack, target_postgres_ack,
    },
    DdlApplyPlanSummary, DdlPlanSummary, Result, TrellaraConfig,
};

pub(crate) fn ddl_release_ack(
    config: &TrellaraConfig,
    plan: &DdlPlanSummary,
    apply_plan: &DdlApplyPlanSummary,
    sink: &str,
    barrier_lsn: &str,
    schema_version: &str,
) -> Result<DdlBarrierAck> {
    match sink {
        "target_postgres" => {
            target_postgres_ack(config, plan, apply_plan, barrier_lsn, schema_version)
        }
        "raw_cdc_lake" => raw_cdc_lake_ack(config, plan, schema_version),
        "spark_derived_views" => spark_derived_views_ack(config, plan, schema_version),
        "partition_visibility" => {
            partition_visibility_ack(config, plan, barrier_lsn, schema_version)
        }
        _ => Ok(generic_sink_ack(config, plan, sink, schema_version)),
    }
}

fn generic_sink_ack(
    config: &TrellaraConfig,
    plan: &DdlPlanSummary,
    sink: &str,
    schema_version: &str,
) -> DdlBarrierAck {
    DdlBarrierAck {
        source_id: config.source.id.clone(),
        database_id: config_source_database_id(config),
        dataset_id: config.dataset.id.clone(),
        barrier_id: plan.propagation.barrier_id.clone(),
        sink: sink.to_string(),
        ack_lsn: SAMPLE_ACK_LSN.to_string(),
        schema_version: schema_version.to_string(),
        accepted: true,
        detail: format!("{sink} accepted schema version {schema_version} after barrier LSN"),
    }
}
