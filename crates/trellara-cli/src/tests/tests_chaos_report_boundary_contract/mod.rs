use super::*;

mod fleet_fanin;
mod partitioned;
mod qualification;
mod repair_replay;
mod snapshot;
mod strict_chunk;

pub(super) fn assert_chaos_report_boundary_contract(summary: &ChaosRunSummary) {
    partitioned::assert_partitioned_barrier_boundaries(summary);
    strict_chunk::assert_strict_chunk_boundaries(summary);
    fleet_fanin::assert_fleet_fanin_boundaries(summary);
    qualification::assert_qualification_boundaries(summary);
    snapshot::assert_snapshot_handoff_boundaries(summary);
    repair_replay::assert_repair_replay_boundaries(summary);
    assert!(summary
        .scenarios
        .iter()
        .all(|scenario| scenario.status == ChaosScenarioStatus::CoveredByTests));
}

fn scenario<'a>(summary: &'a ChaosRunSummary, name: &str) -> &'a ChaosScenarioSummary {
    summary
        .scenarios
        .iter()
        .find(|scenario| scenario.name == name)
        .unwrap_or_else(|| panic!("missing {name} scenario"))
}
