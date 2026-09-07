#[path = "pilot_evidence_ddl_markers/json.rs"]
mod json;

use json::{
    ack_commands_are_executable, ack_evidence_has_valid_sink_lsns, cdc_transaction_boundary,
    has_no_release_blockers, post_ddl_release_gate_satisfied, release_dml_true,
    release_evidence_summarizes_decision,
};

pub(crate) fn ddl_release_marker_present(marker: &str, contents: &str) -> bool {
    if looks_like_json(contents) && !json::release_summary_matches_top_level(contents) {
        return false;
    }

    let lower = contents.to_ascii_lowercase();
    let compact = lower
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .collect::<String>();

    match marker {
        "source dataset identity" => json::source_dataset_identity_present(contents),
        "release_dml true" => {
            release_dml_true(contents)
                || !looks_like_json(contents)
                    && (lower.contains("release_dml: true")
                        || compact.contains("\"release_dml\":true"))
        }
        "post_ddl_dml_release" => post_ddl_release_gate_satisfied(contents),
        "cdc_transaction_boundary" => {
            cdc_transaction_boundary(contents)
                || !looks_like_json(contents) && json::cdc_boundary_holds_post_ddl_dml(contents)
        }
        "ack_commands" => ack_commands_are_executable(contents),
        "ack_evidence" => ack_evidence_has_valid_sink_lsns(contents),
        "release_evidence" => release_evidence_summarizes_decision(contents),
        "ddl_dml_replay_proof" => json::ddl_dml_replay_proof(contents),
        "ddl_propagation_policy" => json::ddl_propagation_policy_proof(contents),
        "propagation_boundary" => json::propagation_boundary_proof(contents),
        "propagation_decisions" => json::propagation_decisions_proof(contents),
        "propagation_policy_sha256" => json::propagation_policy_digest_proof(contents),
        "no release blockers" => has_no_release_blockers(contents),
        _ => lower.contains(&marker.to_ascii_lowercase()),
    }
}

fn looks_like_json(contents: &str) -> bool {
    contents.trim_start().starts_with('{')
}
