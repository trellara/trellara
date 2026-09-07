use sha2::{Digest, Sha256};

use crate::ddl_propagation_policy::ddl_propagation_policy_modes;
use crate::ddl_types::*;
use crate::{
    ddl_cdc_transaction_boundary, ddl_propagation_phases, ddl_propagation_sinks,
    ddl_release_blockers, ddl_release_gates, DatasetMode, TrellaraConfig,
};

pub(crate) fn ddl_propagation_plan(
    config: &TrellaraConfig,
    verdict: DdlPlanVerdict,
    changes: &[DdlPlanChange],
) -> DdlPropagationPlan {
    let requires_global_partition_pause = config.dataset.mode == DatasetMode::PartitionedScaleMode;
    let sinks = ddl_propagation_sinks(config, requires_global_partition_pause);
    let phases = ddl_propagation_phases(verdict);
    let release_gates = ddl_release_gates(config.dataset.mode, verdict, &sinks);
    let release_blockers = ddl_release_blockers(verdict, changes);
    let policy_modes = ddl_propagation_policy_modes(verdict, changes);
    let cdc_transaction_boundary = ddl_cdc_transaction_boundary(config, &policy_modes);

    DdlPropagationPlan {
        barrier_id: ddl_barrier_id(config, changes),
        barrier_scope: ddl_barrier_scope(changes),
        cdc_transaction_boundary,
        row_visibility_mode: ddl_row_visibility_mode(config, verdict),
        dml_after_barrier_held: true,
        requires_global_partition_pause,
        sink_count: sinks.len(),
        required_ack_count: sinks.len(),
        ack_quorum: ddl_ack_quorum(requires_global_partition_pause),
        policy_modes,
        release_blockers,
        sinks,
        phases,
        release_gates,
    }
}

pub(crate) fn ddl_barrier_id(config: &TrellaraConfig, changes: &[DdlPlanChange]) -> String {
    let mut input = format!(
        "{}:{}:{}",
        config.source.id, config.dataset.id, config.dataset.mode
    );
    for change in changes {
        input.push('|');
        input.push_str(&change.input);
    }
    let digest = format!("{:x}", Sha256::digest(input.as_bytes()));
    format!("ddl-barrier-{}", &digest[..12])
}

pub(crate) fn ddl_barrier_scope(changes: &[DdlPlanChange]) -> String {
    let mut relations = changes
        .iter()
        .filter_map(|change| change.relation.clone())
        .collect::<Vec<_>>();
    relations.sort();
    relations.dedup();
    if relations.is_empty() {
        "dataset".to_string()
    } else {
        format!("relations: {}", relations.join(", "))
    }
}

pub(crate) fn ddl_row_visibility_mode(config: &TrellaraConfig, verdict: DdlPlanVerdict) -> String {
    if verdict == DdlPlanVerdict::Blocked
        && config.dataset.mode == DatasetMode::PartitionedScaleMode
    {
        "blocked: later DML remains quarantined; partitioned global visibility stays paused until all partition lanes can acknowledge the schema barrier".to_string()
    } else if verdict == DdlPlanVerdict::Blocked {
        "blocked: later DML remains quarantined until schema policy is fixed".to_string()
    } else if config.dataset.mode == DatasetMode::PartitionedScaleMode {
        "partitioned: all partition lanes must acknowledge the schema barrier before global visibility"
            .to_string()
    } else {
        "strict: later transactions remain invisible until every sink acknowledges the schema barrier"
            .to_string()
    }
}

fn ddl_ack_quorum(requires_global_partition_pause: bool) -> String {
    if requires_global_partition_pause {
        "all_required_sinks_plus_every_partition_lane".to_string()
    } else {
        "all_required_sinks".to_string()
    }
}
