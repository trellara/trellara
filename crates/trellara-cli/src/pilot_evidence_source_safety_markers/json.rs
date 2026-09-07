use serde_json::Value;

use super::{
    acceptable_status, json_path, meaningful_string_field, string_field_eq, string_field_non_empty,
    u64_field,
};

pub(super) fn identity_present(value: &Value) -> bool {
    meaningful_string_field(value, &["source_id"])
        && meaningful_string_field(value, &["dataset_id"])
}

pub(super) fn has_source_safety_shape(value: &Value) -> bool {
    identity_present(value) && status_is_acceptable(value)
}

pub(super) fn has_no_critical_findings(value: &Value) -> bool {
    let Some(factor_count) = u64_field(value, &["factor_count"]) else {
        return false;
    };
    let Some(critical_factor_count) = u64_field(value, &["critical_factor_count"]) else {
        return false;
    };
    let Some(factors) = json_path(value, &["factors"]).and_then(Value::as_array) else {
        return false;
    };

    factor_count == factors.len() as u64
        && critical_factor_count
            == factors
                .iter()
                .filter(|factor| factor_is_critical(factor))
                .count() as u64
        && critical_factor_count == 0
        && factors.iter().all(noncritical_factor_has_evidence)
}

pub(super) fn status_is_acceptable(value: &Value) -> bool {
    json_path(value, &["status"])
        .and_then(Value::as_str)
        .is_some_and(acceptable_status)
}

fn factor_is_critical(factor: &Value) -> bool {
    string_field_eq(factor, &["severity"], "critical")
}

fn noncritical_factor_has_evidence(factor: &Value) -> bool {
    !factor_is_critical(factor)
        && string_field_eq(factor, &["severity"], "warning")
        && string_field_non_empty(factor, &["code"])
        && string_field_non_empty(factor, &["evidence"])
        && string_field_non_empty(factor, &["recommendation"])
}
