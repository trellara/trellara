use serde_json::Value;

use crate::pilot_evidence_identity_helpers::source_dataset_identity_present;

pub(crate) fn contract_preflight_marker_present(marker: &str, contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };

    match marker {
        "source dataset identity" => contract_identity_present(&value),
        "passed true" => contract_passed(&value),
        "checks present" => contract_checks_present(&value),
        "check counts consistent" => contract_check_counts_consistent(&value),
        "no failed or error checks" => contract_has_no_failed_or_error_checks(&value),
        _ => false,
    }
}

fn contract_identity_present(value: &Value) -> bool {
    source_dataset_identity_present(value, &["source_id"], &["dataset_id"])
}

fn contract_passed(value: &Value) -> bool {
    bool_field(value, &["passed"]).is_some_and(|passed| passed)
        && u64_field(value, &["issue_count"]).is_some_and(|issue_count| issue_count == 0)
}

fn contract_checks_present(value: &Value) -> bool {
    checks(value).is_some_and(|checks| {
        !checks.is_empty()
            && checks.iter().all(|check| {
                string_field_non_empty(check, &["name"])
                    && string_field_non_empty(check, &["message"])
                    && bool_field(check, &["passed"]).is_some()
                    && string_field_non_empty(check, &["severity"])
            })
    })
}

fn contract_check_counts_consistent(value: &Value) -> bool {
    let Some(checks) = checks(value) else {
        return false;
    };
    let Some(check_count) = u64_field(value, &["check_count"]) else {
        return false;
    };
    let Some(issue_count) = u64_field(value, &["issue_count"]) else {
        return false;
    };

    let actual_issue_count = checks
        .iter()
        .filter(|check| bool_field(check, &["passed"]) == Some(false))
        .count() as u64;

    check_count == checks.len() as u64 && issue_count == actual_issue_count
}

fn contract_has_no_failed_or_error_checks(value: &Value) -> bool {
    checks(value).is_some_and(|checks| {
        !checks.is_empty()
            && checks.iter().all(|check| {
                bool_field(check, &["passed"]) == Some(true)
                    && !string_field_eq(check, &["severity"], "error")
                    && !string_field_eq(check, &["severity"], "critical")
            })
    })
}

fn checks(value: &Value) -> Option<&Vec<Value>> {
    json_path(value, &["checks"]).and_then(Value::as_array)
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

fn json_value(contents: &str) -> Option<Value> {
    serde_json::from_str(contents).ok()
}
