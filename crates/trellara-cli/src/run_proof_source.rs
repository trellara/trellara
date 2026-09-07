use std::path::Path;

use crate::{BootstrapSummary, RelaySummary, RunProofGate, RunProofStatus};

pub(crate) fn source_bootstrap_position_run_proof(
    config_path: &Path,
    bootstrap: &BootstrapSummary,
) -> RunProofGate {
    let config_display = config_path.display().to_string();
    RunProofGate {
        code: "source_bootstrap_position".to_string(),
        status: if bootstrap.preflight.passed && bootstrap.consistent_lsn.is_some() {
            RunProofStatus::Verified
        } else {
            RunProofStatus::AtRisk
        },
        evidence: format!(
            "publication={} slot={} relation_count={} consistent_lsn={}",
            bootstrap.publication,
            bootstrap.slot,
            bootstrap.relation_count,
            bootstrap.consistent_lsn.as_deref().unwrap_or("missing")
        ),
        proof_command: format!("trellara check --config {config_display} --format text"),
    }
}

pub(crate) fn local_durability_ack_run_proof(
    config_path: &Path,
    relay: &RelaySummary,
) -> RunProofGate {
    let config_display = config_path.display().to_string();
    RunProofGate {
        code: "source_ack_after_local_durability".to_string(),
        status: if relay.source_ack_after_durable_publish {
            RunProofStatus::Verified
        } else {
            RunProofStatus::NeedsEvidence
        },
        evidence: format!(
            "{} transactions published as {} durable local messages; last_topic={} last_offset={} source_ack_lsn={} publish_destinations_match={} publish_destination_count={} boundary={}",
            relay.published_transactions,
            relay.published_messages,
            relay.last_topic.as_deref().unwrap_or("none"),
            relay
                .last_offset
                .map(|offset| offset.to_string())
                .unwrap_or_else(|| "none".to_string()),
            relay.source_ack_lsn.as_deref().unwrap_or("none"),
            relay.source_ack_publish_destinations_match,
            relay.source_ack_publish_destination_count,
            relay
                .source_ack_contract
                .as_deref()
                .unwrap_or("missing structured source ack boundary proof")
        ) + &local_stream_evidence(relay),
        proof_command: format!("trellara stream inspect-local --config {config_display}"),
    }
}

fn local_stream_evidence(relay: &RelaySummary) -> String {
    relay
        .local_stream_evidence
        .as_ref()
        .map(|evidence| {
            format!(
                "; local_stream_status={} total_messages={} pending_messages={} torn_tail_bytes={} rebuilt_index_topics={} unhealthy_cursors={}",
                evidence.status,
                evidence.total_messages,
                evidence.total_pending_messages,
                evidence.torn_tail_bytes,
                evidence.rebuilt_index_topics,
                evidence.unhealthy_cursors
            ) + &last_publish_ack_proof_evidence(evidence)
                + &source_ack_durability_proof_evidence(evidence)
        })
        .unwrap_or_default()
}

fn source_ack_durability_proof_evidence(evidence: &crate::LocalRunStreamEvidence) -> String {
    evidence
        .source_ack_durability_proof
        .as_ref()
        .map(|proof| {
            format!(
                "; source_ack_durability_proof={}{} source_ack_expected_publish_messages={} source_ack_durable_publish_acks={} source_ack_all_publish_acks_proven={} source_ack_proofed_ack_count={} source_ack_proofed_destinations={}",
                proof.contract,
                source_ack_durability_label(proof),
                proof.expected_publish_messages,
                proof.durable_publish_acks,
                proof.all_publish_acks_proven,
                proof.proofed_ack_count,
                source_ack_proofed_destinations(proof)
            )
        })
        .unwrap_or_default()
}

fn source_ack_proofed_destinations(proof: &crate::LocalSourceAckEvidence) -> String {
    if proof.publish_ack_proofs.is_empty() {
        return "none".to_string();
    }
    proof
        .publish_ack_proofs
        .iter()
        .map(|ack| format!("{}:{}:{}", ack.topic, ack.partition, ack.offset))
        .collect::<Vec<_>>()
        .join(",")
}

fn last_publish_ack_proof_evidence(evidence: &crate::LocalRunStreamEvidence) -> String {
    evidence
        .last_publish_ack_proof
        .as_ref()
        .map(|proof| {
            format!(
                "; last_publish_ack_proof={}{} last_publish_ack_topic={} last_publish_ack_partition={} last_publish_ack_offset={} last_publish_ack_key={} last_publish_ack_indexed={} last_publish_ack_replayable={} last_publish_ack_index_status={} last_publish_ack_torn_tail_bytes={}",
                proof.contract,
                publish_ack_durability_label(proof),
                proof.topic,
                proof.partition,
                proof.offset,
                proof.key,
                proof.indexed,
                proof.replayable,
                proof.index_status,
                proof.torn_tail_bytes
            )
        })
        .unwrap_or_default()
}

fn source_ack_durability_label(proof: &crate::LocalSourceAckEvidence) -> String {
    format!(
        " source_ack_durability={} source_ack_crash_safe_ack={}",
        proof.durability, proof.crash_safe_ack
    )
}

fn publish_ack_durability_label(proof: &crate::LocalPublishAckEvidence) -> String {
    format!(
        " last_publish_ack_durability={} last_publish_ack_crash_safe_ack={}",
        proof.durability, proof.crash_safe_ack
    )
}
