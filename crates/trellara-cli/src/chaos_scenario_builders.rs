use crate::chaos_types::*;

pub(crate) struct ChaosScenarioInput {
    pub(crate) name: &'static str,
    pub(crate) failure_point: &'static str,
    pub(crate) invariant: &'static str,
    pub(crate) boundary_mode: &'static str,
    pub(crate) expected_safety_property: &'static str,
    pub(crate) proof_command: &'static str,
    pub(crate) recovery_command: Option<&'static str>,
    pub(crate) evidence: &'static str,
}

impl ChaosScenarioSummary {
    pub(crate) fn covered(input: ChaosScenarioInput) -> Self {
        Self::from_input(ChaosScenarioStatus::CoveredByTests, input)
    }

    fn from_input(status: ChaosScenarioStatus, input: ChaosScenarioInput) -> Self {
        Self {
            name: input.name.to_string(),
            status,
            failure_point: input.failure_point.to_string(),
            invariant: input.invariant.to_string(),
            boundary_mode: input.boundary_mode.to_string(),
            expected_safety_property: input.expected_safety_property.to_string(),
            proof_command: input.proof_command.to_string(),
            recovery_command: input.recovery_command.map(str::to_string),
            evidence: input.evidence.to_string(),
        }
    }
}
