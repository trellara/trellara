use crate::{DatasetMode, KafkaProfile, StreamConfig, TrellaraConfig};

impl TrellaraConfig {
    pub fn explain(&self) -> String {
        let table_count = self.dataset.tables.len();
        let stream = match &self.stream {
            StreamConfig::Kafka {
                bootstrap_servers,
                topic,
                profile,
                ..
            } => format!(
                "kafka profile={} topic={topic} bootstrap_servers={bootstrap_servers}{}",
                profile.name(),
                kafka_durability(profile)
            ),
            StreamConfig::Local { path, .. } => format!("local path={}", path.display()),
        };
        let transaction_boundary = match self.dataset.mode {
            DatasetMode::StrictTransactionOrder => {
                if let Some(strict_chunking) = &self.dataset.strict_chunking {
                    format!(
                        "one source transaction is chunked at {} changes per chunk and applied only after the manifest and commit marker barrier proves completeness",
                        strict_chunking.max_changes_per_chunk
                    )
                } else {
                    "one source transaction is published and applied as one ordered envelope"
                        .to_string()
                }
            }
            DatasetMode::PartitionedScaleMode => {
                let partition = self
                    .dataset
                    .partition
                    .as_ref()
                    .expect("validated partition settings");
                format!(
                    "one source transaction is split across {} partitions by key {}, with manifest and commit marker barriers preserving transaction completeness; null_key_policy={} key_change_policy={}",
                    partition.partition_count,
                    partition.key_column,
                    partition.null_key_policy,
                    partition.key_change_policy
                )
            }
        };

        format!(
            "config_version={} environment={} source={} dataset={} tables={} mode={} stream={}\ntransaction_boundary={}",
            self.config_version,
            self.environment,
            self.source.id,
            self.dataset.id,
            table_count,
            self.dataset.mode,
            stream,
            transaction_boundary
        )
    }
}

fn kafka_durability(profile: &KafkaProfile) -> String {
    match profile {
        KafkaProfile::Development => String::new(),
        KafkaProfile::Production { contract } => format!(
            " replication_factor={} min_insync_replicas={} tls=required",
            contract.replication_factor, contract.min_insync_replicas
        ),
    }
}
