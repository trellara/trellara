use std::collections::BTreeSet;

use trellara_sim::{
    run_default_fleet_fanin_suite, run_default_qualification_suite, run_default_snapshot_suite,
    run_default_strict_chunk_suite, run_default_suite,
};

use crate::chaos_scenarios::default_chaos_scenarios;
use crate::chaos_types::*;
use crate::default_enterprise_review_gates;

impl Default for ChaosRunSummary {
    fn default() -> Self {
        let deterministic_seed = 20_260_812;
        let simulations = run_default_suite(deterministic_seed)
            .into_iter()
            .map(ChaosSimulationSummary::from_report)
            .collect::<Vec<_>>();
        let snapshot_simulations = run_default_snapshot_suite(deterministic_seed)
            .into_iter()
            .map(ChaosSnapshotSimulationSummary::from_report)
            .collect::<Vec<_>>();
        let strict_chunk_simulations = run_default_strict_chunk_suite(deterministic_seed)
            .into_iter()
            .map(ChaosStrictChunkSimulationSummary::from_report)
            .collect::<Vec<_>>();
        let fleet_fanin_simulations = run_default_fleet_fanin_suite(deterministic_seed)
            .into_iter()
            .map(ChaosFleetFanInSimulationSummary::from_report)
            .collect::<Vec<_>>();
        let qualification_simulations = run_default_qualification_suite(deterministic_seed)
            .into_iter()
            .map(ChaosQualificationSimulationSummary::from_report)
            .collect::<Vec<_>>();
        let simulations_passed = simulations.iter().all(|simulation| simulation.passed)
            && snapshot_simulations
                .iter()
                .all(|simulation| simulation.passed)
            && strict_chunk_simulations
                .iter()
                .all(|simulation| simulation.passed)
            && fleet_fanin_simulations
                .iter()
                .all(|simulation| simulation.passed)
            && qualification_simulations
                .iter()
                .all(|simulation| simulation.passed);
        let scenarios = default_chaos_scenarios();
        let covered_scenarios = scenarios
            .iter()
            .filter(|scenario| scenario.status == ChaosScenarioStatus::CoveredByTests)
            .count();
        Self {
            metadata: ChaosReportMetadata::default(),
            mode: "deterministic_failure_matrix".to_string(),
            passed: simulations_passed && covered_scenarios == scenarios.len(),
            scenario_count: scenarios.len(),
            covered_scenarios,
            invariant_count: scenarios
                .iter()
                .map(|scenario| scenario.invariant.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            deterministic_seed,
            simulation_count: simulations.len()
                + snapshot_simulations.len()
                + strict_chunk_simulations.len()
                + fleet_fanin_simulations.len()
                + qualification_simulations.len(),
            simulations_passed,
            simulations,
            snapshot_simulations,
            strict_chunk_simulations,
            fleet_fanin_simulations,
            qualification_simulations,
            scenarios,
            performance_envelope: ChaosPerformanceEnvelope::default(),
            enterprise_review_gates: default_enterprise_review_gates(),
            verification_command: "cargo test --workspace".to_string(),
        }
    }
}
