use super::*;

pub(in crate::tests) fn caught_up_lag() -> CheckpointLag {
    CheckpointLag {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        last_seen_lsn: "0/16B6C50".to_string(),
        last_durable_lsn: "0/16B6C50".to_string(),
        last_applied_lsn: "0/16B6C50".to_string(),
        seen_to_durable_bytes: 0,
        durable_to_applied_bytes: 0,
        seen_to_applied_bytes: 0,
        source_is_durable: true,
        target_is_caught_up: true,
    }
}

pub(in crate::tests) fn target_lag() -> CheckpointLag {
    CheckpointLag {
        last_applied_lsn: "0/16B6B00".to_string(),
        durable_to_applied_bytes: 336,
        target_is_caught_up: false,
        ..caught_up_lag()
    }
}

pub(in crate::tests) fn latest_quarantine() -> ApplyQuarantine {
    ApplyQuarantine {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-blocked".to_string(),
        commit_lsn: "0/16B6D28".to_string(),
        reason: "target_postgres_error".to_string(),
        detail: "target table is missing".to_string(),
        attempt_count: 2,
        last_seen_at: "2026-08-11 10:00:00+00".to_string(),
    }
}

pub(in crate::tests) fn latest_reseed() -> ReseedEvent {
    ReseedEvent {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        watermark_lsn: "0/16B8000".to_string(),
        table_count: 1,
        copied_rows: 12,
        completed_at: "2026-08-11 10:05:00+00".to_string(),
    }
}

pub(in crate::tests) fn latest_snapshot_handoff() -> SnapshotHandoffEvent {
    SnapshotHandoffEvent {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        relation: "public.sales".to_string(),
        watermark_lsn: "0/16B8000".to_string(),
        copied_rows: 12,
        completed_at: "2026-08-11 10:05:00+00".to_string(),
    }
}

pub(in crate::tests) fn latest_snapshot_run() -> SnapshotRun {
    SnapshotRun {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        run_id: "snapshot-run-1".to_string(),
        state: trellara_checkpoint::SnapshotRunState::Streaming,
        slot_name: "slot-a".to_string(),
        consistent_lsn: Some("0/16B8000".to_string()),
        current_relation: Some("public.sales".to_string()),
        copied_rows: 12,
        failure_reason: None,
        started_at: "2026-08-11 10:00:00+00".to_string(),
        updated_at: "2026-08-11 10:05:00+00".to_string(),
    }
}

pub(in crate::tests) fn fixture_snapshot_copy_summary() -> SnapshotCopySummary {
    SnapshotCopySummary {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        run_id: "snapshot-run-1".to_string(),
        state: SnapshotRunState::StreamHandoffReady.to_string(),
        slot: "slot-a".to_string(),
        consistent_lsn: "0/16B8000".to_string(),
        selected_table_count: 1,
        table_count: 1,
        skipped_table_count: 0,
        copied_rows: 12,
        tables: vec![SnapshotCopyTableSummary {
            relation: "public.sales".to_string(),
            state: SnapshotRunState::CopyComplete.to_string(),
            copied_rows: 12,
            skipped: false,
            watermark_lsn: "0/16B8000".to_string(),
        }],
        next_commands: vec!["trellara relay --config <config>".to_string()],
        handoff_proof_command: Some(
            "trellara snapshot --config <config> --run-id snapshot-run-1 --table public.sales"
                .to_string(),
        ),
        consistency_note: "snapshot reached stream handoff".to_string(),
        handoff_blocker_codes: Vec::new(),
        recovery_actions: Vec::new(),
    }
}

pub(in crate::tests) fn latest_validation(converged: bool) -> ValidationEvent {
    ValidationEvent {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        source_watermark_lsn: "0/16B9000".to_string(),
        target_watermark_lsn: "0/16B9000".to_string(),
        converged,
        table_count: 3,
        drift_count: if converged { 0 } else { 2 },
        drift_relations: if converged {
            Vec::new()
        } else {
            vec!["public.sales".to_string(), "public.refunds".to_string()]
        },
        evidence_sha256: Some("a".repeat(64)),
        completed_at: "2026-08-11 10:10:00+00".to_string(),
    }
}

pub(in crate::tests) fn incomplete_partition_watermarks() -> PartitionWatermarkSummary {
    PartitionWatermarkSummary {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        expected_partition_count: 4,
        observed_partition_count: 2,
        complete_partition_set: false,
        global_durable_lsn: None,
        global_applied_lsn: None,
        global_durable_to_applied_bytes: None,
        missing_partitions: vec![1, 3],
        partitions: Vec::new(),
    }
}
