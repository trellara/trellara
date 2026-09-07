use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct QuickstartSummary {
    pub(crate) config: String,
    pub(crate) objective: String,
    pub(crate) estimated_minutes: u32,
    pub(crate) time_budget_minutes: u32,
    pub(crate) evidence_bundle: String,
    pub(crate) recovery_command: String,
    pub(crate) operator_reference_commands: Vec<String>,
    pub(crate) command_count: usize,
    pub(crate) commands: Vec<QuickstartCommand>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct QuickstartCommand {
    pub(crate) step: u32,
    pub(crate) command: String,
    pub(crate) purpose: String,
}

impl QuickstartCommand {
    pub(crate) fn new(step: u32, command: impl Into<String>, purpose: impl Into<String>) -> Self {
        Self {
            step,
            command: command.into(),
            purpose: purpose.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct QuickstartReadinessSummary {
    pub(crate) config: String,
    pub(crate) ready: bool,
    pub(crate) estimated_minutes: Option<u32>,
    pub(crate) time_budget_minutes: u32,
    pub(crate) evidence_bundle: Option<String>,
    pub(crate) recovery_command: Option<String>,
    pub(crate) check_count: usize,
    pub(crate) passed_check_count: usize,
    pub(crate) checks: Vec<QuickstartReadinessCheck>,
    pub(crate) next_commands: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct QuickstartReadinessCheck {
    pub(crate) code: String,
    pub(crate) passed: bool,
    pub(crate) message: String,
    pub(crate) fix: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct MvpReadinessSummary {
    pub(crate) config: String,
    pub(crate) ready: bool,
    pub(crate) criterion_count: usize,
    pub(crate) passed_criterion_count: usize,
    pub(crate) criteria: Vec<MvpReadinessCriterion>,
    pub(crate) priority_next_commands: Vec<String>,
    pub(crate) next_commands: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct MvpReadinessCriterion {
    pub(crate) code: String,
    pub(crate) passed: bool,
    pub(crate) evidence: String,
    pub(crate) proof_command: String,
}
