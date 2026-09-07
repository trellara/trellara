use trellara_protocol::TransactionEnvelope;

use crate::epoch::{
    LakeEpoch, LakeEpochConfig, LakeEpochPartition, LakeEpochSource, LakeEpochSourceState,
    LakeEpochTable, LakeEpochVerification,
};
use crate::epoch_completeness::{epoch_state, mark_missing_sources, verification_status};
use crate::epoch_summary_builder::EpochSummaryBuilder;
use crate::LakeError;

pub fn build_epoch_summary(
    config: &LakeEpochConfig,
    envelopes: &[TransactionEnvelope],
) -> Result<LakeEpoch, LakeError> {
    let mut builder = EpochSummaryBuilder::new(config);
    builder.ingest_all(envelopes)?;
    finalize_epoch_summary(config, builder)
}

fn finalize_epoch_summary(
    config: &LakeEpochConfig,
    mut builder: EpochSummaryBuilder,
) -> Result<LakeEpoch, LakeError> {
    let (missing_source_count, quarantined_source_count) =
        mark_missing_sources(config, &mut builder.sources)?;
    let complete_source_count = builder
        .sources
        .values()
        .filter(|source| source.state == LakeEpochSourceState::Complete)
        .count();
    let state = epoch_state(config, missing_source_count, quarantined_source_count);

    Ok(LakeEpoch {
        epoch_id: config.epoch_id.clone(),
        dataset_id: config.dataset_id.clone(),
        state,
        straggler_policy: config.straggler_policy.clone(),
        required_source_count: config.required_sources.len(),
        complete_source_count,
        missing_source_count,
        quarantined_source_count,
        transaction_count: builder.transaction_count,
        change_count: builder.change_count,
        checksum_rollup: builder.checksum_rollup,
        sources: builder
            .sources
            .into_iter()
            .map(|(source_id, source)| LakeEpochSource {
                source_id,
                state: source.state,
                start_lsn: source.start_lsn,
                end_lsn: source.end_lsn,
                transaction_count: source.transaction_count,
                change_count: source.change_count,
                checksum_rollup: source.checksum_rollup,
                gap_reason: source.gap_reason,
            })
            .collect(),
        tables: builder
            .tables
            .into_iter()
            .map(|(relation, table)| LakeEpochTable {
                relation,
                transaction_count: table.transactions.len(),
                change_count: table.change_count,
                checksum_rollup: table.checksum_rollup,
            })
            .collect(),
        partitions: builder
            .partitions
            .into_iter()
            .map(
                |((source_id, partition_id), partition)| LakeEpochPartition {
                    source_id,
                    partition_id,
                    first_commit_lsn: partition.first_commit_lsn,
                    last_commit_lsn: partition.last_commit_lsn,
                    transaction_count: partition.transaction_count,
                    event_count: partition.event_count,
                    checksum_rollup: partition.checksum_rollup,
                },
            )
            .collect(),
        verification: LakeEpochVerification {
            stream_transaction_count: builder.transaction_count,
            stream_change_count: builder.change_count,
            checksum_rollup: builder.checksum_rollup,
            status: verification_status(state),
        },
    })
}
