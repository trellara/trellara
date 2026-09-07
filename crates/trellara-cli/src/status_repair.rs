use std::path::Path;

use serde::Serialize;

use crate::{
    partition_watermark_repair_steps, validation_drift_repair_steps, validation_stale_repair_steps,
    FlowFailureSummary, FlowStatusSummary,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct RepairPlanSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) dry_run: bool,
    pub(crate) plan_required: bool,
    pub(crate) latest_failure: Option<FlowFailureSummary>,
    pub(crate) step_count: usize,
    pub(crate) steps: Vec<RepairPlanStep>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct RepairPlanStep {
    pub(crate) order: usize,
    pub(crate) action_code: String,
    pub(crate) reason: String,
    pub(crate) command: String,
    pub(crate) redelivery_topics: Vec<String>,
    pub(crate) redelivery_warnings: Vec<String>,
    pub(crate) hint: String,
}

impl RepairPlanSummary {
    pub(crate) fn from_status(status: FlowStatusSummary, config_path: &Path) -> Self {
        let config_path = config_path.display().to_string();
        let mut steps = status
            .recovery_actions
            .iter()
            .flat_map(|action| {
                action
                    .command_templates
                    .iter()
                    .map(|command| (action, command.replace("<config>", &config_path)))
                    .collect::<Vec<_>>()
            })
            .enumerate()
            .map(|(index, (action, command))| RepairPlanStep {
                order: index + 1,
                action_code: action.code.clone(),
                reason: action.reason.clone(),
                command,
                redelivery_topics: action.redelivery_topics.clone(),
                redelivery_warnings: action.redelivery_warnings.clone(),
                hint: action.hint.clone(),
            })
            .collect::<Vec<_>>();
        steps.extend(partition_watermark_repair_steps(
            &status.mode,
            status.partition_watermarks.as_ref(),
            &config_path,
            steps.len(),
        ));
        steps.extend(validation_drift_repair_steps(
            status.latest_validation.as_ref(),
            &config_path,
            steps.len(),
        ));
        steps.extend(validation_stale_repair_steps(
            status.latest_validation.as_ref(),
            status.source.as_ref(),
            status.target.as_ref(),
            &config_path,
            steps.len(),
        ));

        Self {
            source_id: status.source_id,
            dataset_id: status.dataset_id,
            dry_run: true,
            plan_required: !steps.is_empty(),
            latest_failure: status.latest_failure,
            step_count: steps.len(),
            steps,
        }
    }
}
