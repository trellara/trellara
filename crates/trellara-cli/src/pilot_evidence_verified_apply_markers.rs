#[path = "pilot_evidence_verified_apply_markers/json.rs"]
mod json;
#[path = "pilot_evidence_verified_apply_markers/text.rs"]
mod text;

use trellara_checkpoint::{lsn_shape_is_valid, parse_lsn};

pub(crate) fn verified_apply_marker_present(marker: &str, contents: &str) -> bool {
    match marker {
        "source dataset identity" => verified_apply_identity_present(contents),
        "converged true" => verified_apply_converged(contents),
        "checksum match" => verified_apply_checksum_matches(contents),
        "target relation identity" => verified_apply_target_relation_identity(contents),
        _ => false,
    }
}

fn verified_apply_identity_present(contents: &str) -> bool {
    json::json_value(contents)
        .as_ref()
        .is_some_and(json::verified_apply_identity_present)
        || text::verified_apply_identity_present(contents)
}

fn verified_apply_converged(contents: &str) -> bool {
    json::json_value(contents)
        .as_ref()
        .is_some_and(json::verified_apply_converged)
        || text::verified_apply_complete(contents)
}

fn verified_apply_checksum_matches(contents: &str) -> bool {
    json::json_value(contents)
        .as_ref()
        .is_some_and(json::verified_apply_checksum_matches)
        || text::verified_apply_complete(contents)
}

fn verified_apply_target_relation_identity(contents: &str) -> bool {
    json::json_value(contents)
        .as_ref()
        .is_some_and(json::verified_apply_target_relation_identity)
        || text::verified_apply_target_relation_identity(contents)
}

fn lsn_valid(lsn: &str) -> bool {
    lsn_shape_is_valid(lsn) && parse_lsn(lsn) > 0
}
