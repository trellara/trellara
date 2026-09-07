use crate::{
    local_stream_durability_label, DatasetMode, LocalStreamDurability, PerformanceEnvelopeItem,
    SourceCaptureKind, StreamConfig, TrellaraConfig,
};

pub(crate) fn performance_expected_bottlenecks(
    config: &TrellaraConfig,
) -> Vec<PerformanceEnvelopeItem> {
    let mut items = vec![
        PerformanceEnvelopeItem::new(
            "source_wal_retention",
            "slow consumers retain source WAL until source feedback can safely advance",
            "source-safety and status expose WAL posture, safe_wal_size, retained bytes, and slot invalidation risk",
        ),
        PerformanceEnvelopeItem::new(
            "target_apply",
            "target table indexes, constraints, and row filters can dominate apply latency",
            "verify and status metrics compare target watermark catch-up against durable source progress",
        ),
    ];

    if config.source.capture == SourceCaptureKind::PgOutput && config.source.pgoutput.streaming {
        items.push(PerformanceEnvelopeItem::new(
            "streamed_transaction_spill",
            "large pgoutput transactions spill before commit so relay memory is bounded by the configured threshold",
            "chaos run covers pgoutput_streamed_transaction_spills_until_commit",
        ));
    }

    match config.dataset.mode {
        DatasetMode::StrictTransactionOrder if config.dataset.strict_chunking.is_some() => {
            items.push(PerformanceEnvelopeItem::new(
                "strict_manifest_barrier",
                "strict chunking improves memory safety but delays visibility until manifest and commit marker are durable",
                "consistency and inspect-transaction expose strict_chunk_manifest boundaries",
            ));
        }
        DatasetMode::PartitionedScaleMode => {
            items.push(PerformanceEnvelopeItem::new(
                "partition_watermark_lag",
                "global visibility is bounded by the slowest participating partition watermark",
                "partition-watermarks reports missing partitions and global low watermarks",
            ));
        }
        DatasetMode::StrictTransactionOrder => {}
    }

    if matches!(
        config.stream,
        StreamConfig::Local {
            durability: LocalStreamDurability::Fsync,
            ..
        }
    ) {
        items.push(PerformanceEnvelopeItem::new(
            "local_fsync",
            "fsync durability adds write latency but is the default proof posture for crash-safe trials",
            "stream inspect-local reports local log, index, cursor, and torn-tail health",
        ));
    }

    items
}

pub(crate) fn performance_tuning_levers(config: &TrellaraConfig) -> Vec<PerformanceEnvelopeItem> {
    let mut items = vec![
        PerformanceEnvelopeItem::new(
            "stream_spill_threshold_changes",
            "raise only after measuring memory headroom and large-transaction latency",
            format!(
                "current threshold is {} changes with supported maximum {}",
                config
                    .source
                    .stream_spill_threshold_changes
                    .unwrap_or(trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES),
                trellara_pg_capture::MAX_STREAM_SPILL_THRESHOLD_CHANGES
            ),
        ),
        PerformanceEnvelopeItem::new(
            "run_bounds",
            "increase max-transactions and max-messages after the 100/100 proof loop is stable",
            "quickstart and performance proof commands start with conservative local loop bounds",
        ),
    ];

    match config.dataset.mode {
        DatasetMode::StrictTransactionOrder if config.dataset.strict_chunking.is_some() => {
            let max_changes = config
                .dataset
                .strict_chunking
                .as_ref()
                .expect("strict chunking present")
                .max_changes_per_chunk;
            items.push(PerformanceEnvelopeItem::new(
                "strict_chunk_size",
                "larger chunks reduce barrier overhead but increase replay and apply blast radius",
                format!("current max_changes_per_chunk is {max_changes}"),
            ));
        }
        DatasetMode::PartitionedScaleMode => {
            let partition = config
                .dataset
                .partition
                .as_ref()
                .expect("validated partition settings");
            items.push(PerformanceEnvelopeItem::new(
                "partition_count",
                "more partitions increase parallelism potential but raise manifest and watermark coordination cost",
                format!(
                    "current partition_count={} key_column={}",
                    partition.partition_count, partition.key_column
                ),
            ));
        }
        DatasetMode::StrictTransactionOrder => {}
    }

    match &config.stream {
        StreamConfig::Local { durability, .. } => {
            items.push(PerformanceEnvelopeItem::new(
                "local_durability",
                "buffered mode can lower trial latency but should not replace fsync evidence for enterprise pilots",
                format!(
                    "current durability={}",
                    local_stream_durability_label(*durability)
                ),
            ));
        }
        StreamConfig::Kafka { .. } => {
            items.push(PerformanceEnvelopeItem::new(
                "broker_configuration",
                "broker acks, retention, partitions, and replication factor shape throughput outside Trellara",
                "capture broker configuration with the performance artifact before making throughput claims",
            ));
        }
    }

    items
}
