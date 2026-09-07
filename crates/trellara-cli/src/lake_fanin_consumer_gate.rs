use crate::LakeEpochSummary;

pub(crate) fn lake_epoch_consumer_decision(
    summary: &LakeEpochSummary,
    accept_complete_with_gaps: bool,
) -> Result<trellara_lake::LakeEpochConsumerDecision, trellara_lake::LakeError> {
    let epoch = trellara_lake::LakeRawCdcEpochRow {
        epoch_id: summary.epoch_id.clone(),
        dataset_id: summary.dataset_id.clone(),
        state: summary.state,
        policy: summary.straggler_policy.clone(),
        opened_at: "artifact".to_string(),
        sealed_at: "artifact".to_string(),
        required_source_count: summary.required_source_count,
        complete_source_count: summary.complete_source_count,
        missing_source_count: summary.missing_source_count,
        quarantined_source_count: summary.quarantined_source_count,
        transaction_count: summary.transaction_count,
        change_count: summary.change_count,
        checksum_rollup: summary.checksum_rollup,
        manifest_digest: summary.manifest_digest.clone(),
        iceberg_snapshot_id: None,
    };
    let verification = trellara_lake::LakeRawCdcEpochVerificationRow {
        epoch_id: summary.epoch_id.clone(),
        verification_id: trellara_lake::raw_cdc_epoch_verification_id(
            &summary.dataset_id,
            &summary.epoch_id,
            summary.checksum_rollup,
        ),
        input_transaction_count: summary.transaction_count,
        input_change_count: summary.change_count,
        lake_transaction_count: summary.transaction_count,
        lake_change_count: summary.change_count,
        checksum_status: summary.verification_status,
        completed_at: "artifact".to_string(),
    };
    trellara_lake::ensure_lake_epoch_consumable(
        &epoch,
        &verification,
        consumer_options(accept_complete_with_gaps),
    )
}

fn consumer_options(accept_complete_with_gaps: bool) -> trellara_lake::LakeEpochConsumerOptions {
    if accept_complete_with_gaps {
        trellara_lake::LakeEpochConsumerOptions::accepting_complete_with_gaps()
    } else {
        trellara_lake::LakeEpochConsumerOptions::strict()
    }
}
