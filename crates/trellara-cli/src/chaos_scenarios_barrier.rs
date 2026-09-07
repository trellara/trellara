use crate::{
    chaos_scenarios_barrier_applier::barrier_applier_scenarios,
    chaos_scenarios_barrier_integrity::barrier_integrity_scenarios,
    chaos_scenarios_barrier_partitioned::partitioned_barrier_scenarios,
    chaos_scenarios_barrier_strict::strict_chunk_barrier_scenarios, ChaosScenarioSummary,
};

pub(crate) fn barrier_scale_scenarios() -> Vec<ChaosScenarioSummary> {
    let mut scenarios = Vec::new();
    scenarios.extend(partitioned_barrier_scenarios());
    scenarios.extend(strict_chunk_barrier_scenarios());
    scenarios.extend(barrier_integrity_scenarios());
    scenarios.extend(barrier_applier_scenarios());
    scenarios
}
