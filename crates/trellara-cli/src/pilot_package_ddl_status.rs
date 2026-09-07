use trellara_checkpoint::{DdlBarrier, DdlBarrierAck, DdlBarrierSummary};

use crate::{config_source_database_id, DdlPlanSummary, Result, TrellaraConfig};

pub(crate) fn pilot_package_ddl_barrier_status(
    config: &TrellaraConfig,
    plan: &DdlPlanSummary,
) -> Result<DdlBarrierSummary> {
    let acknowledged_sink = plan
        .propagation
        .sinks
        .first()
        .map(|sink| sink.name.clone())
        .unwrap_or_else(|| "raw_cdc_lake".to_string());
    let barrier = DdlBarrier {
        source_id: config.source.id.clone(),
        database_id: config_source_database_id(config),
        dataset_id: config.dataset.id.clone(),
        barrier_id: plan.propagation.barrier_id.clone(),
        barrier_lsn: "0/16B8000".to_string(),
        schema_version: "schema-v2".to_string(),
        cdc_transaction_boundary: plan.propagation.cdc_transaction_boundary.clone(),
        required_sinks: plan
            .propagation
            .sinks
            .iter()
            .map(|sink| sink.name.clone())
            .collect(),
        requires_global_partition_pause: plan.propagation.requires_global_partition_pause,
    };
    let target_ack = DdlBarrierAck {
        source_id: config.source.id.clone(),
        database_id: config_source_database_id(config),
        dataset_id: config.dataset.id.clone(),
        barrier_id: plan.propagation.barrier_id.clone(),
        sink: acknowledged_sink,
        ack_lsn: "0/16B9000".to_string(),
        schema_version: "schema-v2".to_string(),
        accepted: true,
        detail: "sample schema barrier ACK accepted; remaining sinks stay pending".to_string(),
    };

    DdlBarrierSummary::try_from_barrier_and_acks(barrier, vec![target_ack]).map_err(Into::into)
}
