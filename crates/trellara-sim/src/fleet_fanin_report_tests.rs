use super::*;
use trellara_lake::{
    LakeEpochSource, LakeEpochSourceState, LakeEpochVerification, LakeEpochVerificationStatus,
    LakeStragglerPolicy,
};

fn epoch(state: LakeCompletenessState) -> LakeEpoch {
    LakeEpoch {
        epoch_id: "epoch-1".to_string(),
        dataset_id: "retail_sales".to_string(),
        state,
        straggler_policy: LakeStragglerPolicy::PublishWithGaps { grace_ms: 300_000 },
        required_source_count: 12,
        complete_source_count: 9,
        missing_source_count: 3,
        quarantined_source_count: 0,
        transaction_count: 9,
        change_count: 9,
        checksum_rollup: 99,
        sources: epoch_sources(),
        tables: vec![LakeEpochTable {
            relation: "public.sales".to_string(),
            transaction_count: 9,
            change_count: 9,
            checksum_rollup: 99,
        }],
        partitions: Vec::new(),
        verification: LakeEpochVerification {
            stream_transaction_count: 9,
            stream_change_count: 9,
            checksum_rollup: 99,
            status: LakeEpochVerificationStatus::Match,
        },
    }
}

fn epoch_sources() -> Vec<LakeEpochSource> {
    (0..12)
        .map(|index| LakeEpochSource {
            source_id: format!("store-{index:03}"),
            state: if index < 9 {
                LakeEpochSourceState::Complete
            } else {
                LakeEpochSourceState::Missing
            },
            start_lsn: (index < 9).then(|| format!("0/{:X}", 0x16B6C50 + index as u64)),
            end_lsn: (index < 9).then(|| format!("0/{:X}", 0x16B6C50 + index as u64)),
            transaction_count: usize::from(index < 9),
            change_count: usize::from(index < 9),
            checksum_rollup: index as u64,
            gap_reason: (index >= 9).then(|| "source missing at epoch seal".to_string()),
        })
        .collect()
}

#[test]
fn fleet_report_uses_epoch_state_when_no_recovery_state_exists() {
    let report = build_fleet_fanin_report(FleetFanInReportInput {
        seed: 5,
        failure_point: FleetFanInFailurePoint::OfflineStoresPublishWithGaps,
        epoch: epoch(LakeCompletenessState::CompleteWithGaps),
        recovered_from: None,
        duplicate_replay_count: 0,
        passed: true,
        injected_failure: None,
        quarantine_entries: Vec::new(),
        steps: Vec::new(),
    });

    assert!(report.passed);
    assert_eq!(
        report.initial_state,
        LakeCompletenessState::CompleteWithGaps
    );
    assert_eq!(report.recovered_state, None);
    assert_eq!(report.missing_source_count, 3);
    assert_eq!(report.straggler_policy, "publish_with_gaps");
    assert_eq!(report.source_watermarks.len(), 12);
    assert_eq!(report.table_rollups.len(), 1);
    assert!(report.partition_rollups.is_empty());
    assert_eq!(report.table_rollups[0].relation, "public.sales");
    assert_eq!(report.table_rollups[0].transaction_count, 9);
    assert_eq!(
        report
            .source_watermarks
            .iter()
            .filter(|source| source.state == "missing")
            .count(),
        3
    );
}

#[test]
fn fleet_report_records_recovered_epoch_transition() {
    let report = build_fleet_fanin_report(FleetFanInReportInput {
        seed: 5,
        failure_point: FleetFanInFailurePoint::LateStoreRecoveryCompletesEpoch,
        epoch: epoch(LakeCompletenessState::Complete),
        recovered_from: Some(LakeCompletenessState::CompleteWithGaps),
        duplicate_replay_count: 0,
        passed: true,
        injected_failure: None,
        quarantine_entries: Vec::new(),
        steps: Vec::new(),
    });

    assert_eq!(
        report.initial_state,
        LakeCompletenessState::CompleteWithGaps
    );
    assert_eq!(
        report.recovered_state,
        Some(LakeCompletenessState::Complete)
    );
}
