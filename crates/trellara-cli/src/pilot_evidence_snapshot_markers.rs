#[path = "pilot_evidence_snapshot_markers/json.rs"]
mod json;
#[path = "pilot_evidence_snapshot_markers/text.rs"]
mod text;

use trellara_checkpoint::{lsn_shape_is_valid, parse_lsn};

pub(crate) fn snapshot_handoff_marker_present(marker: &str, contents: &str) -> bool {
    let value = json::json_value(contents);

    match marker {
        "source dataset identity" => {
            value
                .as_ref()
                .is_some_and(json::source_dataset_identity_present)
                || text::source_dataset_identity_present(contents)
        }
        "stream_handoff_ready" => {
            value.as_ref().is_some_and(json::snapshot_handoff_ready)
                || text::snapshot_handoff_complete(contents)
        }
        "copy_complete" => {
            value.as_ref().is_some_and(json::snapshot_tables_complete)
                || text::snapshot_handoff_complete(contents)
        }
        "selected table coverage" => {
            value.as_ref().is_some_and(json::selected_table_coverage)
                || text::text_selected_table_coverage(contents)
        }
        "durable handoff watermark" => {
            value.as_ref().is_some_and(json::watermark_consistent)
                || text::snapshot_handoff_complete(contents)
        }
        _ => false,
    }
}

fn lsn_present(value: &str) -> bool {
    let value = value.trim();
    lsn_valid(value)
        && !value.eq_ignore_ascii_case("missing")
        && !value.eq_ignore_ascii_case("none")
        && !value.eq_ignore_ascii_case("null")
}

fn lsn_valid(value: &str) -> bool {
    lsn_shape_is_valid(value) && parse_lsn(value) > 0
}
