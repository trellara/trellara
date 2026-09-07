use crate::{DatasetMode, LocalStreamDurability, SourceCaptureKind, StreamConfig, TrellaraConfig};

pub(crate) fn performance_capture_contract(config: &TrellaraConfig) -> String {
    match config.source.capture {
        SourceCaptureKind::PgOutput => format!(
            "pgoutput protocol_version={} streaming={} with spill threshold {} changes",
            config.source.pgoutput.protocol_version,
            config.source.pgoutput.streaming,
            config
                .source
                .stream_spill_threshold_changes
                .unwrap_or(trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES)
        ),
        SourceCaptureKind::TestDecoding => {
            "test_decoding is a low-credibility performance spike; production measurement should use pgoutput".to_string()
        }
    }
}

pub(crate) fn performance_transaction_boundary_cost(config: &TrellaraConfig) -> String {
    match config.dataset.mode {
        DatasetMode::StrictTransactionOrder if config.dataset.strict_chunking.is_some() => {
            let strict_chunking = config
                .dataset
                .strict_chunking
                .as_ref()
                .expect("strict chunking present");
            format!(
                "strict chunking limits relay memory to chunks of {} changes while adding manifest and commit-marker coordination before visibility",
                strict_chunking.max_changes_per_chunk
            )
        }
        DatasetMode::StrictTransactionOrder => {
            "single-envelope strict mode has the simplest apply path but large transactions depend on pgoutput streaming and spill before publish".to_string()
        }
        DatasetMode::PartitionedScaleMode => {
            let partition = config
                .dataset
                .partition
                .as_ref()
                .expect("validated partition settings");
            format!(
                "partitioned scale fans out across {} partitions by {} while adding manifest, commit-marker, and global watermark coordination for atomic visibility",
                partition.partition_count, partition.key_column
            )
        }
    }
}

pub(crate) fn performance_transport_durability_cost(config: &TrellaraConfig) -> String {
    match &config.stream {
        StreamConfig::Local {
            path, durability, ..
        } => match durability {
            LocalStreamDurability::Fsync => format!(
                "local fsync durability under {} favors crash-safe source acknowledgement over lowest latency",
                path.display()
            ),
            LocalStreamDurability::Buffered => format!(
                "local buffered durability under {} lowers latency for trials but weakens crash evidence",
                path.display()
            ),
        },
        StreamConfig::Kafka {
            bootstrap_servers,
            ..
        } => format!(
            "Kafka/Redpanda durability depends on broker configuration at {bootstrap_servers}; Trellara still waits for complete publish acknowledgement before source feedback"
        ),
    }
}
