use serde_json::Value;
use trellara_checkpoint::{lsn_shape_is_valid, parse_lsn};

use crate::pilot_evidence_identity_helpers::meaningful_json_string;

use super::json_helpers::{
    bool_at_either, json_path, json_value, string_at, string_at_either, u64_at_either,
};

pub(super) fn source_dataset_identity_present(contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };

    string_non_empty_at(Some(&value), &["source_id"])
        && string_non_empty_at(Some(&value), &["dataset_id"])
        || string_non_empty_at(Some(&value), &["transaction_boundary", "source_id"])
            && string_non_empty_at(Some(&value), &["transaction_boundary", "dataset_id"])
}

pub(super) fn transaction_boundary_verified(contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };

    string_at(Some(&value), &["transaction_boundary", "status"])
        .or_else(|| string_at(Some(&value), &["transaction_boundary_status"]))
        .is_some_and(|status| status == "verified")
}

pub(super) fn checkpoint_evidence(contents: &str, checkpoint: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };
    let Some(checkpoint_value) = checkpoint_value(&value, checkpoint) else {
        return false;
    };

    match checkpoint {
        "source_checkpoint" => {
            lsn_field_valid(checkpoint_value, "last_seen_lsn")
                && lsn_field_valid(checkpoint_value, "last_durable_lsn")
        }
        "target_checkpoint" => {
            lsn_field_valid(checkpoint_value, "last_durable_lsn")
                && lsn_field_valid(checkpoint_value, "last_applied_lsn")
                && target_applied_covers_required_checkpoint(&value, checkpoint_value)
        }
        _ => false,
    }
}

pub(super) fn source_ack_lsn_evidence(contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };
    let Some(source_ack_lsn) = source_ack_lsn(&value).filter(|lsn| lsn_valid(lsn)) else {
        return false;
    };
    let Some(durable_lsn) = checkpoint_value(&value, "source_checkpoint")
        .and_then(|checkpoint| string_at(Some(checkpoint), &["last_durable_lsn"]))
        .filter(|lsn| lsn_valid(lsn))
    else {
        return false;
    };

    parse_lsn(source_ack_lsn) >= parse_lsn(durable_lsn)
}

pub(super) fn source_ack_after_durable_publish(contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };

    bool_at_either(&value, "source_ack_after_durable_publish")
        .is_some_and(|ack_after_durable| ack_after_durable)
        && super::json_ack::local_source_ack_durability_proof_present(&value)
        && super::json_ack::local_publish_ack_proof_present(&value)
}

pub(super) fn source_ack_publish_destinations(contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };

    let Some(destination_count) = u64_at_either(&value, "source_ack_publish_destination_count")
    else {
        return false;
    };

    bool_at_either(&value, "source_ack_publish_destinations_match").is_some_and(|matched| matched)
        && destination_count > 0
        && source_ack_destination_count_matches_proof(&value, destination_count)
}

fn source_ack_destination_count_matches_proof(value: &Value, destination_count: u64) -> bool {
    super::json_ack::expected_publish_messages(value)
        .is_none_or(|expected| expected == destination_count)
}

pub(super) fn parallel_replay_contract(contents: &str) -> bool {
    let Some(value) = json_value(contents) else {
        return false;
    };
    string_at_either(&value, "parallel_replay_contract").is_some_and(replay_contract_is_valid)
}

fn checkpoint_value<'a>(value: &'a Value, checkpoint: &str) -> Option<&'a Value> {
    json_path(Some(value), &[checkpoint])
        .or_else(|| json_path(Some(value), &["transaction_boundary", checkpoint]))
}

fn source_ack_lsn(value: &Value) -> Option<&str> {
    string_at(Some(value), &["source_ack_lsn"])
        .or_else(|| string_at(Some(value), &["transaction_boundary", "source_ack_lsn"]))
}

fn lsn_field_valid(value: &Value, key: &str) -> bool {
    string_at(Some(value), &[key]).is_some_and(lsn_valid)
}

fn target_applied_covers_required_checkpoint(root: &Value, value: &Value) -> bool {
    let Some(durable_lsn) = string_at(Some(value), &["last_durable_lsn"]) else {
        return false;
    };
    let Some(applied_lsn) = string_at(Some(value), &["last_applied_lsn"]) else {
        return false;
    };
    let required_lsn = checkpoint_value(root, "source_checkpoint")
        .and_then(|checkpoint| string_at(Some(checkpoint), &["last_durable_lsn"]))
        .filter(|lsn| lsn_valid(lsn))
        .map(parse_lsn)
        .unwrap_or_else(|| parse_lsn(durable_lsn))
        .max(parse_lsn(durable_lsn));
    parse_lsn(applied_lsn) >= required_lsn
}

fn string_non_empty_at(value: Option<&Value>, path: &[&str]) -> bool {
    value.is_some_and(|value| meaningful_json_string(value, path))
}

fn lsn_valid(value: &str) -> bool {
    lsn_shape_is_valid(value) && parse_lsn(value) > 0
}

fn replay_contract_is_valid(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("parallel replay is disabled")
        || (lower.contains("dml-only") && lower.contains("ddl barrier"))
}
