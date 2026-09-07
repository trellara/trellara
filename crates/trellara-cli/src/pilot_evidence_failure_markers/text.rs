use crate::pilot_evidence_identity_helpers::meaningful_str;

pub(super) fn source_dataset_identity_present(lower: &str) -> bool {
    text_key_value_present(lower, "source") && text_key_value_present(lower, "dataset")
}

pub(super) fn diagnostics_shape(lower: &str) -> bool {
    lower.contains("trellara diagnostics")
        && text_key_value_present(lower, "config")
        && text_key_value_present(lower, "mode")
        && text_key_value_present(lower, "status")
        && text_bool_key_present(lower, "ready")
}

pub(super) fn repair_plan_surface(lower: &str) -> bool {
    text_bool_key_present(lower, "repair_plan_required")
        && text_recommended_action_is_actionable(lower)
        && lower.contains("trellara repair-plan --config")
}

pub(super) fn quarantine_surface(lower: &str) -> bool {
    lower.contains("trellara quarantine list --config")
        && (lower.contains("target_quarantine")
            || lower.contains("quarantine list")
            || lower.contains("quarantine replay-ready")
            || lower.contains("attachment_commands:"))
}

fn text_recommended_action_is_actionable(lower: &str) -> bool {
    let Some((_, actions)) = lower.split_once("recommended_actions:") else {
        return false;
    };

    for line in actions.lines() {
        let action = line.trim();
        if action.ends_with(':') {
            break;
        }
        if action.starts_with('-') && action.contains("trellara ") {
            return true;
        }
    }
    false
}

fn text_key_value_present(lower: &str, key: &str) -> bool {
    lower
        .lines()
        .filter_map(|line| line.trim().strip_prefix(&format!("{key}:")))
        .any(meaningful_str)
}

fn text_bool_key_present(lower: &str, key: &str) -> bool {
    lower
        .lines()
        .filter_map(|line| line.trim().strip_prefix(&format!("{key}:")))
        .map(str::trim)
        .any(|value| matches!(value, "true" | "false"))
}
