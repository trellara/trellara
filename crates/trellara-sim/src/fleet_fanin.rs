use crate::fleet_fanin_state::FleetFanInScenario;
pub use crate::fleet_fanin_types::{
    FleetFanInFailurePoint, FleetFanInPartitionRollup, FleetFanInQuarantineEntry,
    FleetFanInSimulationAction, FleetFanInSimulationConfig, FleetFanInSimulationReport,
    FleetFanInSimulationStep, FleetFanInSourceWatermark, FleetFanInTableRollup,
};

pub fn run_fleet_fanin_simulation(
    config: FleetFanInSimulationConfig,
) -> FleetFanInSimulationReport {
    let scenario = FleetFanInScenario::new(config);
    scenario.run()
}

pub fn run_default_fleet_fanin_suite(seed: u64) -> Vec<FleetFanInSimulationReport> {
    FleetFanInFailurePoint::ALL
        .into_iter()
        .enumerate()
        .map(|(index, failure_point)| {
            run_fleet_fanin_simulation(FleetFanInSimulationConfig::new(
                seed.wrapping_add(300 + index as u64),
                failure_point,
            ))
        })
        .collect()
}
