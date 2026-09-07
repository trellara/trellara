use serde_json::Value;

use super::json_helpers::{bool_at, json_path, string_at, u64_at};

const LOCAL_PUBLISH_ACK_PROOF_CONTRACT: &str =
    "local_publish_ack_is_indexed_replayable_and_untorn_before_source_ack";
const LOCAL_SOURCE_ACK_DURABILITY_CONTRACT: &str =
    "every_local_publish_ack_is_indexed_replayable_untorn_and_destination_matched_before_source_ack";

pub(super) fn local_source_ack_durability_proof_present(value: &Value) -> bool {
    local_source_ack_durability_proof_value(value).is_some_and(|proof| {
        string_at(Some(proof), &["contract"])
            .is_some_and(|contract| contract == LOCAL_SOURCE_ACK_DURABILITY_CONTRACT)
            && bool_at(Some(proof), &["all_publish_acks_proven"]).is_some_and(|proven| proven)
            && bool_at(Some(proof), &["crash_safe_ack"]).is_some_and(|safe| safe)
            && matching_positive_counts(proof)
    })
}

pub(super) fn local_publish_ack_proof_present(value: &Value) -> bool {
    local_publish_ack_proof_value(value).is_some_and(|proof| {
        string_at(Some(proof), &["contract"])
            .is_some_and(|contract| contract == LOCAL_PUBLISH_ACK_PROOF_CONTRACT)
            && bool_at(Some(proof), &["indexed"]).is_some_and(|indexed| indexed)
            && bool_at(Some(proof), &["replayable"]).is_some_and(|replayable| replayable)
            && bool_at(Some(proof), &["crash_safe_ack"]).is_some_and(|safe| safe)
            && u64_at(Some(proof), &["torn_tail_bytes"]).is_some_and(|bytes| bytes == 0)
    })
}

fn local_publish_ack_proof_value(value: &Value) -> Option<&Value> {
    json_path(Some(value), &["last_publish_ack_proof"])
        .or_else(|| {
            json_path(
                Some(value),
                &["transaction_boundary", "last_publish_ack_proof"],
            )
        })
        .or_else(|| {
            json_path(
                Some(value),
                &["relay", "local_stream_evidence", "last_publish_ack_proof"],
            )
        })
        .or_else(|| {
            json_path(
                Some(value),
                &[
                    "transaction_boundary",
                    "relay",
                    "local_stream_evidence",
                    "last_publish_ack_proof",
                ],
            )
        })
}

fn local_source_ack_durability_proof_value(value: &Value) -> Option<&Value> {
    json_path(Some(value), &["source_ack_durability_proof"])
        .or_else(|| {
            json_path(
                Some(value),
                &["transaction_boundary", "source_ack_durability_proof"],
            )
        })
        .or_else(|| {
            json_path(
                Some(value),
                &[
                    "relay",
                    "local_stream_evidence",
                    "source_ack_durability_proof",
                ],
            )
        })
        .or_else(|| {
            json_path(
                Some(value),
                &[
                    "transaction_boundary",
                    "relay",
                    "local_stream_evidence",
                    "source_ack_durability_proof",
                ],
            )
        })
}

pub(super) fn expected_publish_messages(value: &Value) -> Option<u64> {
    local_source_ack_durability_proof_value(value)
        .and_then(|proof| u64_at(Some(proof), &["expected_publish_messages"]))
}

fn matching_positive_counts(proof: &Value) -> bool {
    let expected = u64_at(Some(proof), &["expected_publish_messages"]);
    let durable = u64_at(Some(proof), &["durable_publish_acks"]);
    let proofed = u64_at(Some(proof), &["proofed_ack_count"]);
    matches!(
        (expected, durable, proofed),
        (Some(expected), Some(durable), Some(proofed))
            if expected > 0 && expected == durable && durable == proofed
    )
}
