use crate::{DatasetMode, LocalStreamDurability, SourceCaptureKind, StreamConfig, TrellaraConfig};

pub(crate) fn consistency_stream_kind(stream: &StreamConfig) -> String {
    match stream {
        StreamConfig::Kafka { .. } => "kafka".to_string(),
        StreamConfig::Local { durability, .. } => {
            format!("local:{}", local_stream_durability_label(*durability))
        }
    }
}

pub(crate) fn local_stream_durability_label(durability: LocalStreamDurability) -> &'static str {
    match durability {
        LocalStreamDurability::Fsync => "fsync",
        LocalStreamDurability::Buffered => "buffered",
    }
}

pub(crate) fn consistency_source_capture_contract(config: &TrellaraConfig) -> String {
    match config.source.capture {
        SourceCaptureKind::PgOutput => format!(
            "pgoutput protocol_version={} streaming={} is the authoritative CDC source; relation metadata fingerprints define the source schema contract",
            config.source.pgoutput.protocol_version, config.source.pgoutput.streaming
        ),
        SourceCaptureKind::TestDecoding => {
            "test_decoding is a non-production spike path; production consistency evidence requires pgoutput metadata and commit boundaries".to_string()
        }
    }
}

pub(crate) fn consistency_transaction_boundary_contract(config: &TrellaraConfig) -> String {
    match config.dataset.mode {
        DatasetMode::StrictTransactionOrder if config.dataset.strict_chunking.is_some() => {
            let strict_chunking = config
                .dataset
                .strict_chunking
                .as_ref()
                .expect("strict chunking present");
            format!(
                "strict chunked: one source commit may emit chunks of at most {} changes, but visibility requires every chunk plus strict_chunk_manifest and commit marker",
                strict_chunking.max_changes_per_chunk
            )
        }
        DatasetMode::StrictTransactionOrder => {
            "strict: one source commit is one ordered envelope and is visible only after the complete envelope is durable".to_string()
        }
        DatasetMode::PartitionedScaleMode => {
            "partitioned: one source commit may span partition lanes; manifest and commit marker preserve transaction identity and completeness across those lanes".to_string()
        }
    }
}

pub(crate) fn consistency_source_ack_contract(config: &TrellaraConfig) -> String {
    match &config.stream {
        StreamConfig::Local { durability, .. } => match durability {
            LocalStreamDurability::Fsync => {
                "source feedback advances only after the local segment append, offset index update, and cursor-safe durable fsync boundary"
                    .to_string()
            }
            LocalStreamDurability::Buffered => {
                "source feedback advances after local buffered append; this is an explicit lower-durability trial posture and not the default enterprise proof"
                    .to_string()
            }
        },
        StreamConfig::Kafka { .. } => {
            "source feedback advances only after the complete transaction boundary is acknowledged by the configured Kafka/Redpanda publisher"
                .to_string()
        }
    }
}

pub(crate) fn consistency_transport_durability_contract(config: &TrellaraConfig) -> String {
    match &config.stream {
        StreamConfig::Local {
            path, durability, ..
        } => format!(
            "local stream stores durable frames under {} with {} durability; replay uses configured topics plus sidecar offset indexes rebuilt from the log when needed",
            path.display(),
            local_stream_durability_label(*durability)
        ),
        StreamConfig::Kafka {
            bootstrap_servers,
            ..
        } => format!(
            "Kafka/Redpanda transport relies on broker durability at {bootstrap_servers}; Trellara still requires complete manifest or envelope publish before checkpoint acknowledgement"
        ),
    }
}

pub(crate) fn consistency_consumer_visibility_contract(config: &TrellaraConfig) -> String {
    match config.dataset.mode {
        DatasetMode::StrictTransactionOrder if config.dataset.strict_chunking.is_some() => {
            "strict consumers expose a transaction only after the strict chunk manifest and commit marker prove every chunk is present".to_string()
        }
        DatasetMode::StrictTransactionOrder => {
            "strict consumers expose a transaction only after the complete source commit envelope is durable and decoded successfully".to_string()
        }
        DatasetMode::PartitionedScaleMode => {
            "barrier-aware consumers expose global current state only after manifest, commit marker, and complete partition watermarks; partition-local consumers must label lower-latency output as non-atomic for cross-partition transactions".to_string()
        }
    }
}

pub(crate) fn consistency_replay_contract(config: &TrellaraConfig) -> String {
    match config.dataset.mode {
        DatasetMode::StrictTransactionOrder if config.dataset.strict_chunking.is_some() => {
            "replay must redeliver the strict chunks, strict_chunk_manifest, and commit marker for the exact transaction boundary before target checkpoints advance".to_string()
        }
        DatasetMode::StrictTransactionOrder => {
            "replay must redeliver the exact source transaction envelope identified by transaction_id and commit_lsn before target checkpoints advance".to_string()
        }
        DatasetMode::PartitionedScaleMode => {
            "replay must redeliver the manifest, commit marker, and every participating partition message for the exact transaction boundary before global visibility advances".to_string()
        }
    }
}
