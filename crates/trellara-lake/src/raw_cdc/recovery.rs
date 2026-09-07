use crate::raw_cdc_types::LakeRawCdcRecoveryScenario;

pub(super) fn recovery_scenarios(epoch_id: &str) -> Vec<LakeRawCdcRecoveryScenario> {
    [
        scenario(
            "writer_crash_before_data_file_commit",
            "writer exits before every planned raw CDC data file is durable",
            "retry_data_file_intents",
            "missing object key hints and absent epoch metadata",
            "replay planned data-file intents by Trellara idempotency keys before acknowledging source transactions",
        ),
        scenario(
            "writer_crash_after_data_before_epoch_metadata",
            "all raw CDC data files are durable but Iceberg checkpoint receipts or epoch metadata are not visible",
            "discover_durable_raw_cdc_objects_and_checkpoint_receipts_then_publish_metadata",
            "object key hints exist while checkpoint receipt or epoch metadata evidence is absent",
            "discover durable raw CDC objects for the epoch, verify file checksums and Iceberg checkpoint receipts, then publish epoch metadata",
        ),
        scenario(
            "writer_crash_after_epoch_metadata",
            "epoch metadata is visible and the writer is retried",
            "discover_existing_epoch_metadata_before_publish",
            "epoch metadata row already exists for the replayed epoch",
            "verify checkpoint receipts and discover existing epoch metadata before publishing duplicate visibility or acknowledging additional source offsets",
        ),
        scenario(
            "conflicting_duplicate_on_replay",
            "a replay presents the same idempotency key with different checksum evidence",
            "quarantine_and_block_epoch",
            "idempotency key collision with a checksum mismatch",
            "quarantine the conflicting replay and block Spark consumption until the source evidence is reconciled",
        ),
        scenario(
            "verification_mismatch_after_commit",
            "stream-to-lake verification does not match after metadata publication",
            "hold_spark_consumption_until_fanin_verify_matches",
            "verification row reports mismatch for the committed epoch",
            "hold Spark consumption for the epoch, retain raw CDC files, and rerun fan-in verification before release",
        ),
    ]
    .into_iter()
    .map(|mut scenario| {
        scenario.trigger = format!("{} for epoch {epoch_id}", scenario.trigger);
        scenario
    })
    .collect()
}

fn scenario(
    code: &str,
    trigger: &str,
    replay_policy: &str,
    operator_evidence: &str,
    recovery_action: &str,
) -> LakeRawCdcRecoveryScenario {
    LakeRawCdcRecoveryScenario {
        code: code.to_string(),
        trigger: trigger.to_string(),
        replay_policy: replay_policy.to_string(),
        operator_evidence: operator_evidence.to_string(),
        recovery_action: recovery_action.to_string(),
    }
}
