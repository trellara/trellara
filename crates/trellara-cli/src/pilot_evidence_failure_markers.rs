#[path = "pilot_evidence_failure_markers/json.rs"]
mod json;
#[path = "pilot_evidence_failure_markers/text.rs"]
mod text;

pub(crate) fn failure_drill_marker_present(marker: &str, contents: &str) -> bool {
    let value = json::json_value(contents);
    let lower = contents.to_ascii_lowercase();

    match marker {
        "source dataset identity" => {
            value
                .as_ref()
                .is_some_and(json::source_dataset_identity_present)
                || text::source_dataset_identity_present(&lower)
        }
        "Trellara diagnostics" => {
            value.as_ref().is_some_and(json::diagnostics_shape) || text::diagnostics_shape(&lower)
        }
        "repair_plan_required" => {
            value.as_ref().is_some_and(json::repair_plan_surface)
                || text::repair_plan_surface(&lower)
        }
        "quarantine" => {
            value.as_ref().is_some_and(json::quarantine_surface) || text::quarantine_surface(&lower)
        }
        _ => false,
    }
}
