use trellara_apply_postgres::TargetDdlBarrierRequirements;
use trellara_protocol::TransactionEnvelope;

use crate::{ddl_propagation_sinks, DatasetMode, TrellaraConfig};

pub(crate) fn ddl_envelope_barrier_requirements(
    config: &TrellaraConfig,
) -> TargetDdlBarrierRequirements {
    let requires_pause = config.dataset.mode == DatasetMode::PartitionedScaleMode;
    TargetDdlBarrierRequirements::from_required_sinks(
        ddl_propagation_sinks(config, requires_pause)
            .into_iter()
            .map(|sink| sink.name),
        requires_pause,
    )
}

pub(crate) fn ddl_envelope_steps(envelope: &TransactionEnvelope) -> Vec<String> {
    vec![
        "record envelope-derived DDL barrier before target schema apply".to_string(),
        "execute target DDL statements in one target schema transaction".to_string(),
        "record target_postgres ACK at the envelope commit LSN".to_string(),
        "wait for ddl-barrier status to release post-DDL DML".to_string(),
        format!(
            "apply {} DML changes from a replay envelope with ddl_events stripped",
            envelope.changes.len()
        ),
    ]
}
