use super::*;
use crate::{
    LakeEpochPartitionRollup, LakeEpochPartitionSkew, LakeEpochScenario, LakeEpochSourceWatermark,
    LakeEpochWatermarkRollup,
};

#[test]
fn summary_manifest_digest_matches_explicit_digest_input() {
    let summary = LakeEpochSummary {
        source_id: "local-source".to_string(),
        dataset_id: "retail".to_string(),
        mode: "strict_transaction_order".to_string(),
        contract: "fleet_fanin_append_only_raw_cdc_with_epoch_completeness".to_string(),
        fanin_mode: "bounded_epoch".to_string(),
        scenario: LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
        epoch_id: "epoch-1".to_string(),
        state: trellara_lake::LakeCompletenessState::Complete,
        recovered_state: None,
        required_source_count: 1,
        complete_source_count: 1,
        missing_source_count: 0,
        quarantined_source_count: 0,
        transaction_count: 1,
        change_count: 1,
        checksum_rollup: 42,
        duplicate_replay_count: 0,
        straggler_policy: "wait_all_required".to_string(),
        straggler_policy_decision: "all required sources completed".to_string(),
        manifest_digest: String::new(),
        watermark_rollup: LakeEpochWatermarkRollup {
            global_low_watermark_lsn: Some("0/16B6C50".to_string()),
            max_source_watermark_lsn: Some("0/16B6C50".to_string()),
            complete_source_count: 1,
            lagging_source_count: 0,
            missing_source_count: 0,
            quarantined_source_count: 0,
            invalid_lsn_source_count: 0,
            invalid_lsn_sources: Vec::new(),
        },
        source_watermarks: vec![source("store-a", "complete", Some("0/16B6C50"))],
        table_rollups: vec![LakeEpochTableRollup {
            relation: "public.sales".to_string(),
            transaction_count: 1,
            change_count: 1,
            checksum_rollup: 42,
        }],
        partition_rollups: vec![LakeEpochPartitionRollup {
            source_id: "store-a".to_string(),
            partition_id: 0,
            first_commit_lsn: Some("0/16B6C50".to_string()),
            last_commit_lsn: Some("0/16B6C50".to_string()),
            transaction_count: 1,
            event_count: 1,
            checksum_rollup: 42,
        }],
        partition_skew: LakeEpochPartitionSkew {
            participating_partition_count: 1,
            total_event_count: 1,
            min_event_count: 1,
            max_event_count: 1,
            skew_ratio_basis_points: Some(10_000),
            hottest_partition_ids: vec![0],
            coolest_partition_ids: vec![0],
        },
        quarantine_entries: Vec::new(),
        verification_status: trellara_lake::LakeEpochVerificationStatus::Match,
        passed: true,
        injected_failure: None,
        visibility_boundary: "epoch metadata visible after verification".to_string(),
        customer_decision: "released".to_string(),
        proof_command: "cargo test -p trellara-sim fanin".to_string(),
        recommended_next_steps: Vec::new(),
    };
    let explicit = lake_epoch_manifest_digest(
        &LakeEpochManifestDigestInput {
            epoch_id: &summary.epoch_id,
            dataset_id: &summary.dataset_id,
            state: summary.state,
            straggler_policy: &summary.straggler_policy,
            required_source_count: summary.required_source_count,
            complete_source_count: summary.complete_source_count,
            missing_source_count: summary.missing_source_count,
            quarantined_source_count: summary.quarantined_source_count,
            transaction_count: summary.transaction_count,
            change_count: summary.change_count,
        },
        &summary.source_watermarks,
        &summary.table_rollups,
        &summary.partition_rollups,
        &summary.partition_skew,
        &summary.quarantine_entries,
    );

    assert_eq!(lake_epoch_summary_manifest_digest(&summary), explicit);
}

fn source(id: &str, state: &str, end_lsn: Option<&str>) -> LakeEpochSourceWatermark {
    LakeEpochSourceWatermark {
        source_id: id.to_string(),
        state: state.to_string(),
        start_lsn: end_lsn.map(str::to_string),
        end_lsn: end_lsn.map(str::to_string),
        transaction_count: usize::from(end_lsn.is_some()),
        change_count: usize::from(end_lsn.is_some()),
        gap_reason: None,
    }
}
