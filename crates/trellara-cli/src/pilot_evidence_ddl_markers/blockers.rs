use serde_json::Value;

use super::json_path;

pub(super) fn has_no_release_blockers(value: &Value) -> bool {
    blocker_arrays_are_empty(value, true)
}

pub(super) fn no_contradictory_release_blockers(value: &Value) -> bool {
    blocker_arrays_are_empty(value, false)
}

fn blocker_arrays_are_empty(value: &Value, require_field: bool) -> bool {
    let blocker_paths = [
        &["release_blockers"][..],
        &["release_blocker_codes"][..],
        &["release_decision", "blocker_codes"][..],
        &["release_summary", "blocker_codes"][..],
    ];
    let mut saw_blocker_field = false;
    for path in blocker_paths {
        let Some(blockers) = json_path(Some(value), path).and_then(Value::as_array) else {
            continue;
        };
        saw_blocker_field = true;
        if !blockers.is_empty() {
            return false;
        }
    }
    saw_blocker_field || !require_field
}
