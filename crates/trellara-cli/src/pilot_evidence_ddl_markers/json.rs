use serde_json::Value;

use crate::pilot_evidence_identity_helpers::meaningful_json_string;

#[path = "ack.rs"]
mod ack;
#[path = "blockers.rs"]
mod blockers;
#[path = "cdc_boundary.rs"]
mod cdc_boundary;
#[path = "identity.rs"]
mod identity;
#[path = "propagation_policy.rs"]
mod propagation_policy;
#[path = "release_decision.rs"]
mod release_decision;
#[path = "release_evidence.rs"]
mod release_evidence;
#[path = "release_gate.rs"]
mod release_gate;
#[path = "replay_proof.rs"]
mod replay_proof;
#[path = "summary_consistency.rs"]
mod summary_consistency;

pub(super) fn source_dataset_identity_present(contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };

    identity::source_dataset_present(&value)
}

pub(super) fn cdc_transaction_boundary(contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };

    cdc_boundary::transaction_boundary_present(&value)
}

pub(super) fn cdc_boundary_holds_post_ddl_dml(boundary: &str) -> bool {
    cdc_boundary::holds_post_ddl_dml(boundary)
}

pub(super) fn release_dml_true(contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };

    release_decision::release_dml_true(&value)
}

pub(super) fn post_ddl_release_gate_satisfied(contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };
    if !blockers::has_no_release_blockers(&value) {
        return false;
    }

    release_gate::post_ddl_release_gate_satisfied(&value)
}

pub(super) fn ack_commands_are_executable(contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };
    let Some(items) = json_path(Some(&value), &["ack_commands"]).and_then(Value::as_array) else {
        return false;
    };

    !items.is_empty()
        && items.iter().all(ack::command_is_executable)
        && ack::command_collection_is_valid(items, &value)
}

pub(super) fn ack_evidence_has_valid_sink_lsns(contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };
    let Some(items) = json_path(Some(&value), &["ack_evidence"]).and_then(Value::as_array) else {
        return false;
    };

    !items.is_empty()
        && items
            .iter()
            .all(|item| ack::evidence_item_is_valid(item, &value))
        && ack::evidence_covers_required_sinks(items, &value)
}

pub(super) fn release_evidence_summarizes_decision(contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };
    release_evidence::summarizes_decision(&value)
}

pub(super) fn has_no_release_blockers(contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };

    blockers::has_no_release_blockers(&value)
}

pub(super) fn ddl_dml_replay_proof(contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };

    replay_proof::ddl_dml_replay_proof(&value)
}

pub(super) fn ddl_propagation_policy_proof(contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };

    propagation_policy::propagation_policy_proof(&value)
}

pub(super) fn propagation_boundary_proof(contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };

    propagation_policy::propagation_boundary_proof(&value)
}

pub(super) fn propagation_decisions_proof(contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };

    propagation_policy::propagation_decisions_proof(&value)
}

pub(super) fn propagation_policy_digest_proof(contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };

    propagation_policy::propagation_policy_digest_proof(&value)
}

pub(super) fn release_summary_matches_top_level(contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };

    summary_consistency::release_summary_matches_top_level(&value)
}

fn json_path<'a>(value: Option<&'a Value>, path: &[&str]) -> Option<&'a Value> {
    path.iter()
        .try_fold(value?, |current, key| current.get(*key))
}

fn meaningful_string_path(value: &Value, path: &[&str]) -> bool {
    meaningful_json_string(value, path)
}

fn json_value(contents: &str) -> Option<Value> {
    serde_json::from_str(contents).ok()
}
