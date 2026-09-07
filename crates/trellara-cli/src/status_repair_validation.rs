use trellara_checkpoint::{CheckpointLag, ValidationEvent};

use crate::{
    validation_drift_message, validation_stale_message, RepairPlanStep, ValidationProgress,
};

pub(crate) fn validation_drift_repair_steps(
    validation: Option<&ValidationEvent>,
    config_path: &str,
    offset: usize,
) -> Vec<RepairPlanStep> {
    let Some(validation) = validation.filter(|validation| !validation.converged) else {
        return Vec::new();
    };
    let relations = if validation.drift_relations.is_empty() {
        vec![None]
    } else {
        validation
            .drift_relations
            .iter()
            .map(|relation| Some(relation.as_str()))
            .collect::<Vec<_>>()
    };

    relations
        .into_iter()
        .flat_map(|relation| {
            let table_arg = relation
                .map(|relation| format!(" --table {relation}"))
                .unwrap_or_default();
            [
                (
                    "validation_drift_reseed",
                    format!("trellara reseed --config {config_path}{table_arg}"),
                    "reseed the drifted table from the source at the recorded validation boundary",
                ),
                (
                    "validation_drift_verify",
                    format!("trellara verify --config {config_path}{table_arg}"),
                    "rerun checksum validation and confirm the table converges before declaring recovery complete",
                ),
            ]
        })
        .enumerate()
        .map(|(index, (action_code, command, hint))| RepairPlanStep {
            order: offset + index + 1,
            action_code: action_code.to_string(),
            reason: validation_drift_message(validation),
            command,
            redelivery_topics: Vec::new(),
            redelivery_warnings: Vec::new(),
            hint: hint.to_string(),
        })
        .collect()
}

pub(crate) fn validation_stale_repair_steps(
    validation: Option<&ValidationEvent>,
    source: Option<&CheckpointLag>,
    target: Option<&CheckpointLag>,
    config_path: &str,
    offset: usize,
) -> Vec<RepairPlanStep> {
    let Some(validation) = validation.filter(|validation| validation.converged) else {
        return Vec::new();
    };
    let progress = ValidationProgress::from_parts(validation, source, target);
    if progress.is_current {
        return Vec::new();
    }

    vec![RepairPlanStep {
        order: offset + 1,
        action_code: "validation_freshness_verify".to_string(),
        reason: validation_stale_message(progress),
        command: format!("trellara verify --config {config_path}"),
        redelivery_topics: Vec::new(),
        redelivery_warnings: Vec::new(),
        hint:
            "rerun checksum validation after target apply reaches the current checkpoint boundary"
                .to_string(),
    }]
}
