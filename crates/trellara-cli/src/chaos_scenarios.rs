use crate::{
    barrier_scale_scenarios, fleet_fanin_scenarios, late_recovery_scenarios,
    pgoutput_capture_scenarios, qualification_scenarios, snapshot_handoff_scenarios,
    source_and_stream_scenarios, target_apply_scenarios, ChaosScenarioSummary,
};

pub(crate) fn default_chaos_scenarios() -> Vec<ChaosScenarioSummary> {
    let mut scenarios = Vec::new();
    scenarios.extend(source_and_stream_scenarios());
    scenarios.extend(target_apply_scenarios());
    scenarios.extend(pgoutput_capture_scenarios());
    scenarios.extend(snapshot_handoff_scenarios());
    scenarios.extend(barrier_scale_scenarios());
    scenarios.extend(fleet_fanin_scenarios());
    scenarios.extend(qualification_scenarios());
    scenarios.extend(late_recovery_scenarios());
    scenarios
}
