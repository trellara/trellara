use std::collections::BTreeSet;

use serde_json::Value;

use crate::pilot_evidence_json_path::{field_contains, json_path};
use crate::pilot_evidence_lake_writer_rows::row_intents;

pub(super) fn duplicate_replay_accounting_present(value: &Value) -> bool {
    let duplicate_count = value
        .get("duplicate_transaction_count")
        .and_then(Value::as_u64);
    duplicate_replay_evidence_present(value, duplicate_count)
        && value
            .get("recovery_scenarios")
            .and_then(Value::as_array)
            .is_some_and(|scenarios| scenarios.iter().any(replay_policy_is_idempotent))
}

fn duplicate_replay_evidence_present(value: &Value, duplicate_count: Option<u64>) -> bool {
    let Some(evidence) = json_path(value, &["duplicate_replay_evidence"]) else {
        return false;
    };
    duplicate_count_matches(evidence, duplicate_count)
        && json_path(evidence, &["replay_safe"])
            .and_then(Value::as_bool)
            .is_some_and(|replay_safe| replay_safe)
        && positive_evidence_count(evidence, "unique_transaction_count")
        && positive_evidence_count(evidence, "row_intent_count")
        && positive_evidence_count(evidence, "idempotency_key_count")
        && replay_counts_match_row_intents(value, evidence)
        && field_contains(
            evidence,
            &["contract"],
            "conflicting idempotency evidence fails closed",
        )
}

fn duplicate_count_matches(evidence: &Value, duplicate_count: Option<u64>) -> bool {
    duplicate_count.is_some_and(|count| {
        json_path(evidence, &["duplicate_transaction_count"]).and_then(Value::as_u64) == Some(count)
    })
}

fn replay_counts_match_row_intents(value: &Value, evidence: &Value) -> bool {
    let Some(rows) = row_intents(value) else {
        return false;
    };
    let Some(row_count) = u64::try_from(rows.len()).ok() else {
        return false;
    };
    let Some(idempotency_key_count) = unique_idempotency_key_count(rows) else {
        return false;
    };

    json_path(evidence, &["row_intent_count"]).and_then(Value::as_u64) == Some(row_count)
        && json_path(evidence, &["idempotency_key_count"]).and_then(Value::as_u64)
            == Some(idempotency_key_count)
}

fn unique_idempotency_key_count(rows: &[Value]) -> Option<u64> {
    let mut keys = BTreeSet::new();
    for row in rows {
        let key = row.get("idempotency_key").and_then(Value::as_str)?.trim();
        if key.is_empty() {
            return None;
        }
        keys.insert(key.to_string());
    }
    u64::try_from(keys.len()).ok()
}

fn positive_evidence_count(evidence: &Value, field: &str) -> bool {
    json_path(evidence, &[field])
        .and_then(Value::as_u64)
        .is_some_and(|count| count > 0)
}

fn replay_policy_is_idempotent(scenario: &Value) -> bool {
    scenario
        .get("replay_policy")
        .and_then(Value::as_str)
        .is_some_and(|policy| policy.contains("discover_existing_epoch_metadata_before_publish"))
}
