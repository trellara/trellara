use std::collections::BTreeSet;

use trellara_lake::LakeRawCdcEpochWritePlan;

use crate::{
    IcebergEpochCommitPlan, IcebergIntegrationError, IcebergMetadataTableKind,
    IcebergTableAppendPlan, Result,
};

pub(super) fn require_expected_files(
    write_plan: &LakeRawCdcEpochWritePlan,
    seen: &BTreeSet<IcebergMetadataTableKind>,
) -> Result<()> {
    let metadata = &write_plan.epoch_metadata;
    let expected = [
        (
            !metadata.source_rows.is_empty(),
            IcebergMetadataTableKind::Source,
        ),
        (
            !metadata.table_rows.is_empty(),
            IcebergMetadataTableKind::Table,
        ),
        (
            !metadata.partition_rows.is_empty(),
            IcebergMetadataTableKind::Partition,
        ),
        (
            !metadata.quarantine_rows.is_empty(),
            IcebergMetadataTableKind::Quarantine,
        ),
        (true, IcebergMetadataTableKind::Verification),
        (true, IcebergMetadataTableKind::Completeness),
    ];
    let missing_tables = expected
        .into_iter()
        .filter(|(required, kind)| *required && !seen.contains(kind))
        .map(|(_, kind)| metadata_table_name(write_plan, kind).to_string())
        .collect::<Vec<_>>();
    if missing_tables.is_empty() {
        Ok(())
    } else {
        Err(IcebergIntegrationError::EpochMetadataNotReady {
            epoch_id: write_plan.epoch_id.clone(),
            missing_tables,
        })
    }
}

pub(super) fn metadata_epoch_plan(
    raw: &IcebergEpochCommitPlan,
    tables: Vec<IcebergTableAppendPlan>,
    visibility_rule: &str,
) -> Result<IcebergEpochCommitPlan> {
    let file_count = tables
        .iter()
        .try_fold(0usize, |count, table| count.checked_add(table.file_count))
        .ok_or(IcebergIntegrationError::CountOverflow {
            field: "metadata.file_count",
        })?;
    let record_count = tables
        .iter()
        .try_fold(0u64, |count, table| count.checked_add(table.record_count))
        .ok_or(IcebergIntegrationError::CountOverflow {
            field: "metadata.record_count",
        })?;
    Ok(IcebergEpochCommitPlan {
        dataset_id: raw.dataset_id.clone(),
        epoch_id: raw.epoch_id.clone(),
        epoch_commit_id: raw.epoch_commit_id.clone(),
        manifest_digest: raw.manifest_digest.clone(),
        checksum_rollup: raw.checksum_rollup,
        transaction_count: raw.transaction_count,
        change_count: raw.change_count,
        file_count,
        record_count,
        accepted_complete_with_gaps: raw.accepted_complete_with_gaps,
        epoch_visibility_rule: visibility_rule.to_string(),
        tables,
    })
}

pub(super) fn metadata_kind(kind: IcebergMetadataTableKind) -> &'static str {
    match kind {
        IcebergMetadataTableKind::Source => "epoch_sources",
        IcebergMetadataTableKind::Table => "epoch_tables",
        IcebergMetadataTableKind::Partition => "epoch_partitions",
        IcebergMetadataTableKind::Quarantine => "quarantine",
        IcebergMetadataTableKind::Verification => "verification",
        IcebergMetadataTableKind::Completeness => "epoch_completeness",
    }
}

fn metadata_table_name(
    write_plan: &LakeRawCdcEpochWritePlan,
    kind: IcebergMetadataTableKind,
) -> &str {
    let metadata = &write_plan.epoch_metadata;
    match kind {
        IcebergMetadataTableKind::Source => &metadata.epoch_sources_table,
        IcebergMetadataTableKind::Table => &metadata.epoch_tables_table,
        IcebergMetadataTableKind::Partition => &metadata.epoch_partitions_table,
        IcebergMetadataTableKind::Quarantine => &metadata.quarantine_table,
        IcebergMetadataTableKind::Verification => &metadata.verification_table,
        IcebergMetadataTableKind::Completeness => &metadata.epochs_table,
    }
}
