#[path = "qualification_labels.rs"]
mod labels;
#[path = "qualification_paths.rs"]
mod paths;
#[path = "qualification_state.rs"]
mod state;
#[path = "qualification_types.rs"]
mod types;

use state::QualificationReportBuilder;

pub use types::{
    QualificationFailurePoint, QualificationObservabilityAssertion, QualificationSimulationAction,
    QualificationSimulationConfig, QualificationSimulationReport, QualificationSimulationStep,
};

pub fn run_qualification_simulation(
    config: QualificationSimulationConfig,
) -> QualificationSimulationReport {
    let mut builder = QualificationReportBuilder::new(config);
    builder.run();
    builder.finish()
}

pub fn run_default_qualification_suite(seed: u64) -> Vec<QualificationSimulationReport> {
    QualificationFailurePoint::ALL
        .into_iter()
        .enumerate()
        .map(|(index, failure_point)| {
            run_qualification_simulation(QualificationSimulationConfig::new(
                seed.wrapping_add(400 + index as u64),
                failure_point,
            ))
        })
        .collect()
}
