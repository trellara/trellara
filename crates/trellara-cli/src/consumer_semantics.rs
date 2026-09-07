use crate::{
    exact_transaction_semantics_mode, partition_local_semantics_mode,
    ConsumerSemanticsAvailability, ConsumerSemanticsMode, DatasetMode, TrellaraConfig,
};

pub(crate) fn consumer_semantics_matrix(
    config: &TrellaraConfig,
    config_display: &str,
) -> Vec<ConsumerSemanticsMode> {
    let partitioned = config.dataset.mode == DatasetMode::PartitionedScaleMode;
    vec![
        exact_transaction_semantics_mode(
            partitioned,
            config.dataset.strict_chunking.is_some(),
            config_display,
        ),
        partition_local_semantics_mode(partitioned, config_display),
        ConsumerSemanticsMode {
            mode: "analytics_append".to_string(),
            best_for: "raw CDC audit tables, search indexing, cache refresh, and webhook/event fanout"
                .to_string(),
            availability: ConsumerSemanticsAvailability::Advisory,
            transaction_boundary:
                "transaction metadata is present; consumer may append before enforcing atomic visibility"
                    .to_string(),
            ordering: if partitioned {
                "per key or partition best effort, with manifest available for completeness checks"
                    .to_string()
            } else {
                "source transaction order available, but consumer may intentionally relax exposure"
                    .to_string()
            },
            latency: "very_low".to_string(),
            throughput: "very_high".to_string(),
            visibility_rule:
                "append immediately, then use transaction id, commit LSN, and manifest metadata for completeness columns"
                    .to_string(),
            consumer_obligations: vec![
                "store idempotency keys with appended rows".to_string(),
                "make partial or pending transaction state queryable instead of pretending it is complete"
                    .to_string(),
            ],
            not_guaranteed: vec![
                "atomic read-your-own-transaction visibility".to_string(),
                "downstream exactly-once side effects".to_string(),
            ],
            proof_commands: vec![
                format!("trellara lake inspect --config {config_display} --file <envelope.pb>"),
                "trellara inspect-transaction --file <envelope.pb> --format text".to_string(),
            ],
        },
        ConsumerSemanticsMode {
            mode: "current_state".to_string(),
            best_for: "serving projections, current-state lake tables, and ML feature reads"
                .to_string(),
            availability: if partitioned {
                ConsumerSemanticsAvailability::SupportedWithBarrier
            } else {
                ConsumerSemanticsAvailability::Native
            },
            transaction_boundary: if partitioned {
                "global low watermark derived from complete partition watermarks".to_string()
            } else {
                "source commit LSN of the complete transaction envelope".to_string()
            },
            ordering: if partitioned {
                "per key ordering, with epoch visibility at the global low watermark".to_string()
            } else {
                "global source transaction order".to_string()
            },
            latency: "medium".to_string(),
            throughput: "high".to_string(),
            visibility_rule: if partitioned {
                "advance table visibility only to the lowest complete partition watermark".to_string()
            } else {
                "advance table visibility to the latest verified source commit LSN".to_string()
            },
            consumer_obligations: vec![
                "persist the Trellara commit LSN with each materialized version".to_string(),
                "run verification before declaring convergence".to_string(),
            ],
            not_guaranteed: vec![
                "visibility beyond the latest verified checkpoint".to_string(),
                "automatic repair without reseed or replay when verification detects drift"
                    .to_string(),
            ],
            proof_commands: vec![
                format!("trellara lake ddl --config {config_display}"),
                format!("trellara verify --config {config_display}"),
            ],
        },
        ConsumerSemanticsMode {
            mode: "lakehouse_epoch".to_string(),
            best_for: "Iceberg raw CDC, Trellara completeness metadata, and Spark-derived current/SCD2 tables"
                .to_string(),
            availability: ConsumerSemanticsAvailability::Advisory,
            transaction_boundary: if partitioned {
                "complete up to global low watermark across partition manifests".to_string()
            } else {
                "complete up to source commit LSN across strict transaction envelopes".to_string()
            },
            ordering: "epoch or commit-LSN window".to_string(),
            latency: "medium".to_string(),
            throughput: "high".to_string(),
            visibility_rule: "publish a new lake snapshot only after the epoch boundary is complete"
                .to_string(),
            consumer_obligations: vec![
                "make the epoch boundary explicit in table metadata".to_string(),
                "do not expose current/SCD2 tables past the verified commit or watermark".to_string(),
            ],
            not_guaranteed: vec![
                "production Iceberg writer availability in the current MVP".to_string(),
                "small-file compaction or table maintenance automation".to_string(),
            ],
            proof_commands: vec![
                format!("trellara lake plan --config {config_display}"),
                format!("trellara lake ddl --config {config_display}"),
            ],
        },
    ]
}

pub(crate) fn consumer_semantics_next_steps(
    config: &TrellaraConfig,
    config_display: &str,
) -> Vec<String> {
    let mut steps = vec![
        format!("trellara contract-test --config {config_display}"),
        "trellara inspect-transaction --file <envelope.pb> --format text".to_string(),
    ];
    if config.dataset.mode == DatasetMode::PartitionedScaleMode {
        steps.push(format!(
            "trellara partition-local --config {config_display}"
        ));
        steps.push(format!(
            "trellara partition-watermarks --config {config_display}"
        ));
    }
    steps.push(format!("trellara lake ddl --config {config_display}"));
    steps
}
