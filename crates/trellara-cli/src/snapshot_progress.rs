use std::collections::HashMap;

use trellara_checkpoint::{SnapshotRunState, SnapshotTableProgress};

use crate::{i64_to_u64_count, Result, SnapshotCopyTableSummary};

pub(crate) fn snapshot_progress_by_relation(
    progress: Vec<SnapshotTableProgress>,
) -> HashMap<String, SnapshotTableProgress> {
    progress
        .into_iter()
        .map(|progress| (progress.relation.clone(), progress))
        .collect()
}

pub(crate) fn skipped_snapshot_table_summary(
    relation: String,
    progress: &SnapshotTableProgress,
) -> Result<SnapshotCopyTableSummary> {
    Ok(SnapshotCopyTableSummary {
        relation,
        state: SnapshotRunState::CopyComplete.to_string(),
        copied_rows: i64_to_u64_count("snapshot copied_rows", progress.copied_rows)?,
        skipped: true,
        watermark_lsn: progress.watermark_lsn.clone().unwrap_or_default(),
    })
}
