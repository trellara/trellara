use crate::{DatasetMode, StreamConfig, TrellaraConfig};

pub(crate) fn enterprise_recommended_mode(config: &TrellaraConfig) -> String {
    match config.dataset.mode {
        DatasetMode::StrictTransactionOrder if config.dataset.strict_chunking.is_some() => {
            "strict_chunked_transaction_order".to_string()
        }
        DatasetMode::StrictTransactionOrder => "strict_transaction_order".to_string(),
        DatasetMode::PartitionedScaleMode => "partitioned_scale_mode".to_string(),
    }
}

pub(crate) fn enterprise_run_proof_command(config_path: &str, local_stream: bool) -> String {
    if local_stream {
        format!("trellara run --local --config {config_path} --verify --format text")
    } else {
        format!("trellara run --config {config_path} --verify --format text")
    }
}

pub(crate) fn enterprise_mode_contract(config: &TrellaraConfig) -> String {
    match config.dataset.mode {
        DatasetMode::StrictTransactionOrder if config.dataset.strict_chunking.is_some() => {
            "Use strict mode when the buyer needs source transaction order by default; large transactions are chunked but remain invisible until every chunk, the strict_chunk_manifest, and the commit marker are durable.".to_string()
        }
        DatasetMode::StrictTransactionOrder => {
            "Use strict mode when the buyer needs the simplest correctness story: one committed source transaction is published and applied as one ordered envelope.".to_string()
        }
        DatasetMode::PartitionedScaleMode => {
            let partition = config
                .dataset
                .partition
                .as_ref()
                .expect("validated partition settings");
            format!(
                "Use partitioned scale mode for high-volume flows naturally keyed by {}; Trellara preserves transaction identity, completeness, and per-key ordering, while barrier-aware consumers wait for the manifest and every partition watermark before atomic visibility.",
                partition.key_column
            )
        }
    }
}

pub(crate) fn enterprise_transaction_boundary_contract(config: &TrellaraConfig) -> String {
    match config.dataset.mode {
        DatasetMode::StrictTransactionOrder if config.dataset.strict_chunking.is_some() => {
            "Transaction boundary: source commit order remains authoritative; manifest `boundary_mode=strict_chunked_transaction_order` proves every strict chunk belongs to the committed transaction before apply.".to_string()
        }
        DatasetMode::StrictTransactionOrder => {
            "Transaction boundary: source commit order remains authoritative; each committed transaction is applied atomically as one envelope.".to_string()
        }
        DatasetMode::PartitionedScaleMode => {
            "Transaction boundary: manifest `boundary_mode=partitioned_scale_mode` preserves identity and completeness across partitions; atomic global visibility requires barrier-aware apply, while partition-local analytics intentionally trade global ordering for lower latency.".to_string()
        }
    }
}

pub(crate) fn enterprise_buyer_summary(
    config: &TrellaraConfig,
    recommended_mode: &str,
    has_target: bool,
) -> String {
    let target = if has_target {
        "target convergence can be verified"
    } else {
        "target convergence is blocked until target.database_url is configured"
    };
    let stream = if matches!(config.stream, StreamConfig::Local { .. }) {
        "brokerless local evaluation"
    } else {
        "Kafka or Redpanda-backed evaluation"
    };

    format!(
        "{recommended_mode} is the configured fit for source={} dataset={} with {}; {}.",
        config.source.id, config.dataset.id, stream, target
    )
}
