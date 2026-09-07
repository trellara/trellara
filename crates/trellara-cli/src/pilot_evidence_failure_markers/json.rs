use serde_json::Value;

use crate::pilot_evidence_identity_helpers::source_dataset_identity_present as json_source_dataset_identity_present;

pub(super) fn source_dataset_identity_present(value: &Value) -> bool {
    json_source_dataset_identity_present(value, &["source_id"], &["dataset_id"])
}

pub(super) fn diagnostics_shape(value: &Value) -> bool {
    string_field_non_empty(value, &["mode"])
        && string_field_non_empty(value, &["config"])
        && string_field_non_empty(value, &["status"])
        && bool_field(value, &["ready"]).is_some()
        && json_path(value, &["report"]).is_some()
}

pub(super) fn repair_plan_surface(value: &Value) -> bool {
    let Some(repair_plan) = json_path(value, &["repair_plan"]) else {
        return false;
    };
    let Some(plan_required) = bool_field(repair_plan, &["plan_required"]) else {
        return false;
    };
    let Some(step_count) = u64_field(repair_plan, &["step_count"]) else {
        return false;
    };
    let Some(steps) = json_path(repair_plan, &["steps"]).and_then(Value::as_array) else {
        return false;
    };

    string_field_non_empty(repair_plan, &["source_id"])
        && string_field_non_empty(repair_plan, &["dataset_id"])
        && bool_field(repair_plan, &["dry_run"]).is_some()
        && repair_steps_match_plan(step_count, plan_required, steps)
        && recommended_actions_present(value)
}

pub(super) fn quarantine_surface(value: &Value) -> bool {
    json_string_array_contains(value, &["attachment_commands"], "trellara quarantine list")
        && (json_path(value, &["report", "no_target_quarantine"])
            .and_then(Value::as_bool)
            .is_some()
            || json_path(value, &["report", "proof_checks"])
                .and_then(Value::as_array)
                .is_some_and(|checks| {
                    checks
                        .iter()
                        .any(|check| string_field_eq(check, &["code"], "target_quarantine"))
                }))
}

pub(super) fn json_value(contents: &str) -> Option<Value> {
    serde_json::from_str(contents).ok()
}

fn repair_steps_match_plan(step_count: u64, plan_required: bool, steps: &[Value]) -> bool {
    step_count == steps.len() as u64
        && (!plan_required || !steps.is_empty())
        && steps.iter().all(repair_step_actionable)
}

fn repair_step_actionable(step: &Value) -> bool {
    string_field_non_empty(step, &["action_code"]) && string_field_non_empty(step, &["command"])
}

fn recommended_actions_present(value: &Value) -> bool {
    json_path(value, &["report", "recommended_actions"])
        .and_then(Value::as_array)
        .is_some_and(|actions| !actions.is_empty())
}

fn json_string_array_contains(value: &Value, path: &[&str], needle: &str) -> bool {
    json_path(value, path)
        .and_then(Value::as_array)
        .is_some_and(|items| {
            items.iter().any(|item| {
                item.as_str()
                    .is_some_and(|actual| actual.to_ascii_lowercase().contains(needle))
            })
        })
}

fn bool_field(value: &Value, path: &[&str]) -> Option<bool> {
    json_path(value, path).and_then(Value::as_bool)
}

fn string_field_eq(value: &Value, path: &[&str], expected: &str) -> bool {
    json_path(value, path)
        .and_then(Value::as_str)
        .is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
}

fn string_field_non_empty(value: &Value, path: &[&str]) -> bool {
    json_path(value, path)
        .and_then(Value::as_str)
        .is_some_and(|actual| !actual.trim().is_empty())
}

fn u64_field(value: &Value, path: &[&str]) -> Option<u64> {
    json_path(value, path).and_then(Value::as_u64)
}

fn json_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter()
        .try_fold(value, |current, key| current.get(*key))
}
