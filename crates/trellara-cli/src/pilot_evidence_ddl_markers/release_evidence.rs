use serde_json::Value;

use super::{ack, blockers};

pub(super) fn summarizes_decision(value: &Value) -> bool {
    let Some(items) = value.get("release_evidence").and_then(Value::as_array) else {
        return false;
    };

    let evidence = items
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    if evidence.is_empty() {
        return false;
    }

    summarizes_barrier_context(&evidence)
        && blockers::no_contradictory_release_blockers(value)
        && summarizes_expected_field(&evidence, value, "barrier_id")
        && summarizes_expected_field(&evidence, value, "barrier_lsn")
        && summarizes_expected_field(&evidence, value, "schema_version")
        && summarizes_expected_field(&evidence, value, "cdc_transaction_boundary")
        && summarizes_ack_count(&evidence)
        && summarizes_required_sink_count(&evidence, value)
        && summarizes_partition_digest_when_required(&evidence, value)
        && release_decision_is_structurally_released(value)
        && summarizes_release_decision(&evidence)
}

fn summarizes_barrier_context(evidence: &[String]) -> bool {
    evidence.iter().any(|item| {
        item.contains("barrier") && item.contains("lsn") && item.contains("schema_version")
    })
}

fn summarizes_expected_field(evidence: &[String], value: &Value, field: &str) -> bool {
    let Some(expected) = value.get(field).and_then(Value::as_str) else {
        return false;
    };
    let expected = expected.to_ascii_lowercase();

    !expected.trim().is_empty() && evidence.iter().any(|item| item.contains(&expected))
}

fn summarizes_ack_count(evidence: &[String]) -> bool {
    evidence
        .iter()
        .any(|item| item.contains("required sink ack") && item.contains("accepted"))
}

fn summarizes_required_sink_count(evidence: &[String], value: &Value) -> bool {
    let Some(required_sinks) = value.get("required_sinks").and_then(Value::as_array) else {
        return false;
    };
    let expected_count = required_sinks
        .iter()
        .filter(|sink| sink.as_str().is_some())
        .count();
    if expected_count == 0 {
        return false;
    }

    let expected_summary = format!("{expected_count}/{expected_count}");
    structured_ack_evidence_matches_required_sinks(value)
        && evidence.iter().any(|item| {
            item.contains(&expected_summary)
                && item.contains("required sink ack")
                && item.contains("accepted")
        })
}

fn structured_ack_evidence_matches_required_sinks(value: &Value) -> bool {
    let Some(items) = value.get("ack_evidence").and_then(Value::as_array) else {
        return false;
    };

    !items.is_empty()
        && items
            .iter()
            .all(|item| ack::evidence_item_is_valid(item, value))
        && ack::evidence_covers_required_sinks(items, value)
}

fn summarizes_release_decision(evidence: &[String]) -> bool {
    evidence.iter().any(|item| {
        item.contains("release_dml=true")
            && (item.contains("blocker_codes=none") || item.contains("blocker_codes=[]"))
    })
}

fn release_decision_is_structurally_released(value: &Value) -> bool {
    value
        .get("release_decision")
        .and_then(|decision| decision.get("release_dml"))
        .and_then(Value::as_bool)
        == Some(true)
}

fn summarizes_partition_digest_when_required(evidence: &[String], value: &Value) -> bool {
    if !requires_partition_visibility(value) {
        return true;
    }
    evidence.iter().any(|item| {
        item.contains("partition_visibility")
            && detail_token(item, "partition_watermark_sha256").is_some_and(sha256_is_valid)
    })
}

fn requires_partition_visibility(value: &Value) -> bool {
    value
        .get("required_sinks")
        .and_then(Value::as_array)
        .is_some_and(|sinks| {
            sinks
                .iter()
                .filter_map(Value::as_str)
                .any(|sink| sink == "partition_visibility")
        })
}

fn detail_token<'a>(detail: &'a str, key: &str) -> Option<&'a str> {
    detail.split(';').map(str::trim).find_map(|token| {
        token
            .strip_prefix(key)
            .and_then(|suffix| suffix.strip_prefix('='))
            .map(str::trim)
    })
}

fn sha256_is_valid(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
}
