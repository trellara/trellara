use crate::{DatasetMode, TrellaraConfig};

pub(crate) fn quickstart_transaction_boundary_message(config: &TrellaraConfig) -> String {
    match config.dataset.mode {
        DatasetMode::StrictTransactionOrder => {
            if let Some(strict_chunking) = &config.dataset.strict_chunking {
                format!(
                    "strict chunk manifest and commit marker boundary enabled at {} changes per chunk",
                    strict_chunking.max_changes_per_chunk
                )
            } else {
                "strict single-envelope boundary enabled; add dataset.strict_chunking for manifest and commit marker barriers on large transactions"
                    .to_string()
            }
        }
        DatasetMode::PartitionedScaleMode => {
            let partition = config
                .dataset
                .partition
                .as_ref()
                .expect("validated partition config");
            format!(
                "partitioned manifest boundary enabled across {} partitions",
                partition.partition_count
            )
        }
    }
}

pub(crate) fn quickstart_capture_spill_message(config: &TrellaraConfig) -> String {
    let threshold = config
        .source
        .stream_spill_threshold_changes
        .unwrap_or(trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES);
    match &config.source.stream_spill_dir {
        Some(path) => format!(
            "pgoutput streamed transaction changes spill after {threshold} changes into {}",
            path.display()
        ),
        None => format!(
            "pgoutput streamed transaction changes spill after {threshold} changes using the OS temp directory"
        ),
    }
}
