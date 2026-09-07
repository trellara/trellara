use std::fmt::Write as _;

use crate::{
    lake_completeness_state_label, lake_epoch_source_state_label,
    lake_epoch_verification_status_label,
};

pub(crate) fn push_epoch_metadata(
    output: &mut String,
    plan: &trellara_lake::LakeRawCdcEpochWritePlan,
) {
    writeln!(output, "epoch_metadata:").expect("write string");
    writeln!(
        output,
        "- epochs_table={} sources_table={} tables_table={} partitions_table={} quarantine_table={} verification_table={}",
        plan.epoch_metadata.epochs_table,
        plan.epoch_metadata.epoch_sources_table,
        plan.epoch_metadata.epoch_tables_table,
        plan.epoch_metadata.epoch_partitions_table,
        plan.epoch_metadata.quarantine_table,
        plan.epoch_metadata.verification_table
    )
    .expect("write string");
    writeln!(
        output,
        "- state={} policy={} sources={}/{} missing={} quarantined={} manifest_digest={} iceberg_snapshot_id={} verification={} verification_id={} completed_at={} source_rows={} table_rows={} partition_rows={} quarantine_rows={}",
        lake_completeness_state_label(plan.epoch_metadata.epoch_row.state),
        plan.epoch_metadata.epoch_row.policy,
        plan.epoch_metadata.epoch_row.complete_source_count,
        plan.epoch_metadata.epoch_row.required_source_count,
        plan.epoch_metadata.epoch_row.missing_source_count,
        plan.epoch_metadata.epoch_row.quarantined_source_count,
        plan.epoch_metadata.epoch_row.manifest_digest,
        plan.epoch_metadata
            .epoch_row
            .iceberg_snapshot_id
            .as_deref()
            .unwrap_or("pending_catalog_commit"),
        lake_epoch_verification_status_label(plan.epoch_metadata.verification_row.checksum_status),
        plan.epoch_metadata.verification_row.verification_id,
        plan.epoch_metadata.verification_row.completed_at,
        plan.epoch_metadata.source_rows.len(),
        plan.epoch_metadata.table_rows.len(),
        plan.epoch_metadata.partition_rows.len(),
        plan.epoch_metadata.quarantine_rows.len()
    )
    .expect("write string");
    push_source_rows(output, plan);
    push_partition_rows(output, plan);
    push_quarantine_rows(output, plan);
}

fn push_source_rows(output: &mut String, plan: &trellara_lake::LakeRawCdcEpochWritePlan) {
    writeln!(
        output,
        "source_rows: count={}",
        plan.epoch_metadata.source_rows.len()
    )
    .expect("write string");
    for source in &plan.epoch_metadata.source_rows {
        writeln!(
            output,
            "- source_row source={} state={} start_lsn={} end_lsn={} transactions={} changes={} checksum={} lag_reason={}",
            source.source_id,
            lake_epoch_source_state_label(source.state),
            source.start_lsn,
            source.end_lsn,
            source.transaction_count,
            source.change_count,
            source.checksum_rollup,
            source.lag_reason.as_deref().unwrap_or("<none>")
        )
        .expect("write string");
    }
}

fn push_partition_rows(output: &mut String, plan: &trellara_lake::LakeRawCdcEpochWritePlan) {
    writeln!(
        output,
        "partition_rows: count={}",
        plan.epoch_metadata.partition_rows.len()
    )
    .expect("write string");
    for partition in &plan.epoch_metadata.partition_rows {
        writeln!(
            output,
            "- partition_row source={} partition={} first_commit_lsn={} last_commit_lsn={} transactions={} events={} checksum={}",
            partition.source_id,
            partition.partition_id,
            partition.first_commit_lsn,
            partition.last_commit_lsn,
            partition.transaction_count,
            partition.event_count,
            partition.checksum_rollup
        )
        .expect("write string");
    }
}

fn push_quarantine_rows(output: &mut String, plan: &trellara_lake::LakeRawCdcEpochWritePlan) {
    writeln!(
        output,
        "quarantine_rows: count={}",
        plan.epoch_metadata.quarantine_rows.len()
    )
    .expect("write string");
    for quarantine in &plan.epoch_metadata.quarantine_rows {
        writeln!(
            output,
            "- quarantine_row source={} tx={} commit_lsn={} reason={} details={} recovery_command={}",
            quarantine.source_id.as_deref().unwrap_or("<none>"),
            quarantine.transaction_id.as_deref().unwrap_or("<none>"),
            quarantine.commit_lsn.as_deref().unwrap_or("<none>"),
            quarantine.reason,
            quarantine.details.as_deref().unwrap_or("<none>"),
            quarantine.recovery_command.as_deref().unwrap_or("<none>")
        )
        .expect("write string");
    }
}
