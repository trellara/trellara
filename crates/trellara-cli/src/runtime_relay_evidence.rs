use trellara_stream::PublishAck;
#[cfg(feature = "local-stream")]
use trellara_stream_local::{
    local_publish_ack_proof_with_durability, local_source_ack_durability_proof_with_durability,
    LocalDurability,
};

#[cfg(feature = "local-stream")]
use crate::{
    inspect_configured_local_stream, CliError, LocalPublishAckEvidence, LocalRunStreamEvidence,
    LocalSourceAckEvidence, Result, StreamConfig, TrellaraConfig,
};
#[cfg(not(feature = "local-stream"))]
use crate::{LocalRunStreamEvidence, Result, TrellaraConfig};

#[cfg(feature = "local-stream")]
pub(crate) fn collect_local_run_stream_evidence(
    config: &TrellaraConfig,
    latest_publish_messages: &[trellara_stream::StreamMessage],
    latest_publish_acks: &[PublishAck],
    last_topic: Option<&str>,
    last_partition: Option<i32>,
    last_offset: Option<i64>,
) -> Result<LocalRunStreamEvidence> {
    let inspection = inspect_configured_local_stream(config)?;
    let (local_path, durability) = local_stream_config(config)?;
    let source_ack_durability_proof = if latest_publish_messages.is_empty() {
        None
    } else {
        let proof = local_source_ack_durability_proof_with_durability(
            local_path,
            latest_publish_messages,
            latest_publish_acks,
            latest_publish_messages.len(),
            durability,
        )?;
        Some(LocalSourceAckEvidence {
            contract: proof.contract.to_string(),
            durability: proof.durability.to_string(),
            crash_safe_ack: proof.crash_safe_ack,
            expected_publish_messages: proof.expected_publish_messages,
            durable_publish_acks: proof.durable_publish_acks,
            all_publish_acks_proven: proof.all_publish_acks_proven,
            proofed_ack_count: proof.ack_proofs.len(),
            publish_ack_proofs: proof
                .ack_proofs
                .into_iter()
                .map(local_publish_ack_evidence)
                .collect(),
        })
    };
    let last_publish_ack_proof = match (last_topic, last_partition, last_offset) {
        (Some(topic), Some(partition), Some(offset)) => {
            let ack = PublishAck {
                topic: topic.to_string(),
                partition,
                offset,
            };
            let proof = local_publish_ack_proof_with_durability(local_path, &ack, durability)?;
            Some(local_publish_ack_evidence(proof))
        }
        _ => None,
    };
    Ok(LocalRunStreamEvidence {
        status: inspection.health.status,
        total_messages: inspection.health.total_messages,
        total_pending_messages: inspection.health.total_pending_messages,
        torn_tail_bytes: inspection.health.torn_tail_bytes,
        rebuilt_index_topics: inspection.health.rebuilt_index_topics,
        unhealthy_cursors: inspection.health.unhealthy_cursors,
        source_ack_durability_proof,
        last_publish_ack_proof,
    })
}

#[cfg(not(feature = "local-stream"))]
pub(crate) fn collect_local_run_stream_evidence(
    _config: &TrellaraConfig,
    _latest_publish_messages: &[trellara_stream::StreamMessage],
    _latest_publish_acks: &[PublishAck],
    _last_topic: Option<&str>,
    _last_partition: Option<i32>,
    _last_offset: Option<i64>,
) -> Result<LocalRunStreamEvidence> {
    Err(crate::local_stream_feature_disabled())
}

#[cfg(feature = "local-stream")]
fn local_publish_ack_evidence(
    proof: trellara_stream_local::LocalPublishAckProof,
) -> LocalPublishAckEvidence {
    LocalPublishAckEvidence {
        contract: proof.contract.to_string(),
        durability: proof.durability.to_string(),
        crash_safe_ack: proof.crash_safe_ack,
        topic: proof.topic,
        partition: proof.partition,
        offset: proof.offset,
        key: proof.key,
        indexed: proof.indexed,
        replayable: proof.replayable,
        index_status: proof.index_status.to_string(),
        torn_tail_bytes: proof.torn_tail_bytes,
    }
}

#[cfg(feature = "local-stream")]
fn local_stream_config(config: &TrellaraConfig) -> Result<(&std::path::Path, LocalDurability)> {
    match &config.stream {
        StreamConfig::Local {
            path, durability, ..
        } => Ok((path.as_path(), (*durability).into())),
        StreamConfig::Kafka { .. } => Err(CliError::InvalidConfig(
            "local run stream evidence requires stream.kind: local".to_string(),
        )),
    }
}
