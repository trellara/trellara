use crate::{ConsumerSemanticsAvailability, ConsumerSemanticsMode};

pub(crate) fn partition_local_semantics_mode(
    partitioned: bool,
    config_display: &str,
) -> ConsumerSemanticsMode {
    ConsumerSemanticsMode {
        mode: "partition_local".to_string(),
        best_for: "store, tenant, account, or entity-lane projections that prefer low latency"
            .to_string(),
        availability: if partitioned {
            ConsumerSemanticsAvailability::Native
        } else {
            ConsumerSemanticsAvailability::NotRecommended
        },
        transaction_boundary: if partitioned {
            "partition chunk carries transaction metadata; manifest remains required for full transaction completeness"
                .to_string()
        } else {
            "not a separate contract in strict mode".to_string()
        },
        ordering: if partitioned {
            "per partition and per ownership key".to_string()
        } else {
            "use exact_transaction instead; strict mode already preserves global order".to_string()
        },
        latency: "low".to_string(),
        throughput: "high".to_string(),
        visibility_rule: if partitioned {
            "a lane may process its own chunk before global transaction completeness is visible"
                .to_string()
        } else {
            "not recommended because partition lanes are not configured".to_string()
        },
        consumer_obligations: vec![
            "treat multi-partition transactions as incomplete unless the manifest barrier is checked"
                .to_string(),
            "honor null-key and key-change policies from the flow config".to_string(),
        ],
        not_guaranteed: vec![
            "atomic visibility for cross-partition transactions".to_string(),
            "global source transaction order".to_string(),
        ],
        proof_commands: vec![
            format!("trellara partition-local --config {config_display}"),
            format!("trellara partition-watermarks --config {config_display}"),
        ],
    }
}
