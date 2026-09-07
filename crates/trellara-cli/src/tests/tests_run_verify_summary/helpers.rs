use super::*;

pub(super) fn verified_bootstrap_summary() -> BootstrapSummary {
    BootstrapSummary {
        publication: "trellara_publication".to_string(),
        slot: "trellara_slot".to_string(),
        consistent_lsn: Some("0/16B8000".to_string()),
        exported_snapshot_name: Some("00000003-0000001B-1".to_string()),
        relation_count: 1,
        relations: vec!["public.sales".to_string()],
        preflight: PreflightSummary {
            passed: true,
            issue_count: 0,
            tables: Vec::new(),
        },
    }
}

pub(super) fn at_risk_bootstrap_summary() -> BootstrapSummary {
    BootstrapSummary {
        publication: "trellara_publication".to_string(),
        slot: "trellara_slot".to_string(),
        consistent_lsn: None,
        exported_snapshot_name: None,
        relation_count: 1,
        relations: vec!["public.sales".to_string()],
        preflight: PreflightSummary {
            passed: false,
            issue_count: 1,
            tables: Vec::new(),
        },
    }
}

pub(super) fn relay_summary(published: u64) -> RelaySummary {
    RelaySummary {
        published_transactions: published,
        published_messages: published,
        last_commit_lsn: Some("0/16B9000".to_string()),
        last_topic: Some("trellara.local-source.sales.strict".to_string()),
        last_partition: Some(0),
        last_offset: published.checked_sub(1).map(|offset| offset as i64),
        source_ack_contract: Some(trellara_relay::SOURCE_ACK_BOUNDARY_CONTRACT.to_string()),
        source_ack_lsn: Some("0/16B9000".to_string()),
        source_ack_after_durable_publish: published > 0,
        source_ack_publish_destinations_match: published > 0,
        source_ack_publish_destination_count: usize::from(published > 0),
        latest_publish_messages: Vec::new(),
        latest_publish_acks: Vec::new(),
        local_stream_evidence: Some(LocalRunStreamEvidence {
            status: "clean".to_string(),
            total_messages: published as i64,
            total_pending_messages: 0,
            torn_tail_bytes: 0,
            rebuilt_index_topics: 0,
            unhealthy_cursors: 0,
            source_ack_durability_proof: (published > 0).then(|| LocalSourceAckEvidence {
                contract: trellara_stream_local::LOCAL_SOURCE_ACK_DURABILITY_CONTRACT.to_string(),
                durability: "fsync".to_string(),
                crash_safe_ack: true,
                expected_publish_messages: published as usize,
                durable_publish_acks: published as usize,
                all_publish_acks_proven: true,
                proofed_ack_count: published as usize,
                publish_ack_proofs: (0..published)
                    .map(|offset| LocalPublishAckEvidence {
                        contract: trellara_stream_local::LOCAL_PUBLISH_ACK_PROOF_CONTRACT
                            .to_string(),
                        durability: "fsync".to_string(),
                        crash_safe_ack: true,
                        topic: "trellara.local-source.sales.strict".to_string(),
                        partition: 0,
                        offset: offset as i64,
                        key: format!("local-source:0/16B9000:tx-{offset}:1"),
                        indexed: true,
                        replayable: true,
                        index_status: "healthy".to_string(),
                        torn_tail_bytes: 0,
                    })
                    .collect(),
            }),
            last_publish_ack_proof: published.checked_sub(1).map(|offset| {
                LocalPublishAckEvidence {
                    contract: trellara_stream_local::LOCAL_PUBLISH_ACK_PROOF_CONTRACT.to_string(),
                    durability: "fsync".to_string(),
                    crash_safe_ack: true,
                    topic: "trellara.local-source.sales.strict".to_string(),
                    partition: 0,
                    offset: offset as i64,
                    key: format!("local-source:0/16B9000:tx-{offset}:1"),
                    indexed: true,
                    replayable: true,
                    index_status: "healthy".to_string(),
                    torn_tail_bytes: 0,
                }
            }),
        }),
    }
}

pub(super) fn apply_summary(applied: u64) -> ApplySummary {
    ApplySummary {
        applied_transactions: applied,
        applied_changes: applied * 2,
        acked_messages: applied,
        last_commit_lsn: Some("0/16B9000".to_string()),
        ..ApplySummary::default()
    }
}

pub(super) fn barrier_pending_apply_summary() -> ApplySummary {
    ApplySummary {
        applied_transactions: 1,
        applied_changes: 2,
        acked_messages: 3,
        last_commit_lsn: Some("0/16B9000".to_string()),
        barrier_pending: ApplyBarrierPendingSummary {
            transactions: 1,
            with_manifest: 1,
            with_commit_marker: 0,
            invalid_commit_marker: 0,
            missing_manifest: 0,
            missing_commit_marker: 1,
            complete_chunk_sets: 1,
            expected_chunks: 1,
            buffered_chunks: 1,
            missing_chunks: 0,
            extra_chunks: 0,
            buffered_messages: 2,
            blockers: vec!["1 transaction(s) missing commit marker".to_string()],
            blocker_codes: vec!["missing_commit_marker".to_string()],
            recovery_actions: vec![
                "replay commit-marker topic for pending barrier transactions before target apply"
                    .to_string(),
            ],
        },
        barrier_pending_transactions: 1,
        barrier_pending_with_manifest: 1,
        barrier_pending_with_commit_marker: 0,
        barrier_pending_invalid_commit_marker: 0,
        barrier_pending_missing_manifest: 0,
        barrier_pending_missing_commit_marker: 1,
        barrier_pending_complete_chunk_sets: 1,
        barrier_pending_expected_chunks: 1,
        barrier_pending_buffered_chunks: 1,
        barrier_pending_missing_chunks: 0,
        barrier_pending_extra_chunks: 0,
        barrier_pending_buffered_messages: 2,
        ..ApplySummary::default()
    }
}

pub(super) fn matching_verify_summary() -> VerifySummary {
    VerifySummary {
        source_watermark_lsn: "0/16B9000".to_string(),
        target_watermark_lsn: "0/16B9000".to_string(),
        converged: true,
        checksum_status: ChecksumStatus::Match,
        tables: vec![matching_table_verify_summary()],
    }
}

pub(super) fn mismatched_verify_summary() -> VerifySummary {
    VerifySummary {
        source_watermark_lsn: "0/16B9000".to_string(),
        target_watermark_lsn: "0/16B8000".to_string(),
        converged: false,
        checksum_status: ChecksumStatus::Mismatch,
        tables: vec![mismatched_table_verify_summary()],
    }
}

pub(super) fn matching_table_verify_summary() -> TableVerifySummary {
    TableVerifySummary {
        relation: "public.sales".to_string(),
        target_relation: "public.sales".to_string(),
        relation_match: true,
        row_filter: None,
        converged: true,
        checksum_status: ChecksumStatus::Match,
        source_row_count: 12,
        target_row_count: 12,
        source_checksum: 99,
        target_checksum: 99,
        missing_in_target_count: 0,
        extra_in_target_count: 0,
        mismatched_row_count: 0,
        drift_sample_limit: 10,
        missing_in_target: Vec::new(),
        extra_in_target: Vec::new(),
        mismatched_rows: Vec::new(),
        evidence_sha256: "a".repeat(64),
        recommended_action: "no action; table converged at the verified watermark".to_string(),
    }
}

fn mismatched_table_verify_summary() -> TableVerifySummary {
    TableVerifySummary {
        relation: "public.sales".to_string(),
        target_relation: "archive.sales".to_string(),
        relation_match: false,
        converged: false,
        checksum_status: ChecksumStatus::Mismatch,
        ..matching_table_verify_summary()
    }
}

pub(super) fn gate<'a>(proof_chain: &'a [RunProofGate], code: &str) -> &'a RunProofGate {
    proof_chain
        .iter()
        .find(|gate| gate.code == code)
        .unwrap_or_else(|| panic!("missing {code} gate"))
}
