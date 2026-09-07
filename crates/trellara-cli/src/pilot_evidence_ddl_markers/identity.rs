use serde_json::Value;

use super::meaningful_string_path;

pub(super) fn source_dataset_present(value: &Value) -> bool {
    top_level_identity_present(value) || release_summary_identity_present(value)
}

fn top_level_identity_present(value: &Value) -> bool {
    meaningful_string_path(value, &["source_id"]) && meaningful_string_path(value, &["dataset_id"])
}

fn release_summary_identity_present(value: &Value) -> bool {
    meaningful_string_path(value, &["release_summary", "source_id"])
        && meaningful_string_path(value, &["release_summary", "dataset_id"])
}
