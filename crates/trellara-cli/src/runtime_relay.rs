use crate::{checked_runtime_add, CliError, RelaySummary, Result};

impl RelaySummary {
    pub(crate) fn from_stats(stats: trellara_relay::RelayRunStats) -> Self {
        Self {
            published_transactions: stats.published_transactions,
            published_messages: stats.published_messages,
            last_commit_lsn: stats.last_commit_lsn,
            last_topic: stats.last_ack.as_ref().map(|ack| ack.topic.clone()),
            last_partition: stats.last_ack.as_ref().map(|ack| ack.partition),
            last_offset: stats.last_ack.map(|ack| ack.offset),
            source_ack_contract: stats
                .last_source_ack_boundary
                .as_ref()
                .map(|proof| proof.contract.to_string()),
            source_ack_lsn: stats
                .last_source_ack_boundary
                .as_ref()
                .map(|proof| proof.source_ack_lsn.clone()),
            source_ack_after_durable_publish: stats.last_source_ack_boundary.as_ref().is_some_and(
                |proof| {
                    proof.durable_lsn_covers_commit
                        && proof.checkpoint_recorded_before_source_ack
                        && proof.all_publish_acks_durable
                        && source_ack_publish_destinations_match(proof)
                },
            ),
            source_ack_publish_destinations_match: stats
                .last_source_ack_boundary
                .as_ref()
                .is_some_and(source_ack_publish_destinations_match),
            source_ack_publish_destination_count: stats
                .last_source_ack_boundary
                .as_ref()
                .map_or(0, |proof| proof.expected_publish_destination_count),
            latest_publish_messages: stats.last_published_messages,
            latest_publish_acks: stats.last_publish_acks,
            local_stream_evidence: None,
        }
    }

    pub(crate) fn record_step(&mut self, step: trellara_relay::RelayStep) -> Result<()> {
        self.published_transactions =
            checked_runtime_add(self.published_transactions, 1, "published_transactions")?;
        let published_messages =
            u64::try_from(step.publish_acks.len()).map_err(|_| CliError::RuntimeStatOverflow {
                field: "published_messages",
            })?;
        self.published_messages = checked_runtime_add(
            self.published_messages,
            published_messages,
            "published_messages",
        )?;
        self.last_commit_lsn = Some(step.envelope.commit_lsn);
        if let Some(ack) = step.publish_acks.last() {
            self.last_topic = Some(ack.topic.clone());
            self.last_partition = Some(ack.partition);
            self.last_offset = Some(ack.offset);
        }
        let proof = &step.source_ack_boundary;
        let destinations_match = source_ack_publish_destinations_match(proof);
        self.source_ack_contract = Some(proof.contract.to_string());
        self.source_ack_lsn = Some(proof.source_ack_lsn.clone());
        self.source_ack_after_durable_publish = proof.durable_lsn_covers_commit
            && proof.checkpoint_recorded_before_source_ack
            && proof.all_publish_acks_durable
            && destinations_match;
        self.source_ack_publish_destinations_match = destinations_match;
        self.source_ack_publish_destination_count = proof.expected_publish_destinations.len();
        self.latest_publish_messages = step.published_messages;
        self.latest_publish_acks = step.publish_acks;
        Ok(())
    }
}

fn source_ack_publish_destinations_match(proof: &trellara_relay::SourceAckBoundaryProof) -> bool {
    proof.expected_publish_destination_count > 0 && proof.publish_destinations_match
}

#[cfg(test)]
#[path = "tests/tests_runtime_relay.rs"]
mod tests;
