use crate::{
    snapshot_validation::{validate_snapshot_run_record, validate_snapshot_table_progress_record},
    Result, SnapshotRun, SnapshotRunState, SnapshotTableProgress,
};

pub(crate) struct SnapshotRunParts {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) run_id: String,
    pub(crate) state: SnapshotRunState,
    pub(crate) slot_name: String,
    pub(crate) consistent_lsn: Option<String>,
    pub(crate) current_relation: Option<String>,
    pub(crate) copied_rows: i64,
    pub(crate) failure_reason: Option<String>,
    pub(crate) started_at: String,
    pub(crate) updated_at: String,
}

pub(crate) fn snapshot_run_from_parts(parts: SnapshotRunParts) -> Result<SnapshotRun> {
    let run = SnapshotRun {
        source_id: parts.source_id,
        dataset_id: parts.dataset_id,
        run_id: parts.run_id,
        state: parts.state,
        slot_name: parts.slot_name,
        consistent_lsn: parts.consistent_lsn,
        current_relation: parts.current_relation,
        copied_rows: parts.copied_rows,
        failure_reason: parts.failure_reason,
        started_at: parts.started_at,
        updated_at: parts.updated_at,
    };
    validate_snapshot_run_record(&run)?;
    Ok(run)
}

pub(crate) fn snapshot_run_from_row(row: tokio_postgres::Row) -> Result<SnapshotRun> {
    snapshot_run_from_parts(SnapshotRunParts {
        source_id: row.get(0),
        dataset_id: row.get(1),
        run_id: row.get(2),
        state: row.get::<_, String>(3).parse()?,
        slot_name: row.get(4),
        consistent_lsn: row.get(5),
        current_relation: row.get(6),
        copied_rows: row.get(7),
        failure_reason: row.get(8),
        started_at: row.get(9),
        updated_at: row.get(10),
    })
}

pub(crate) struct SnapshotTableProgressParts {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) run_id: String,
    pub(crate) relation: String,
    pub(crate) state: SnapshotRunState,
    pub(crate) copied_rows: i64,
    pub(crate) watermark_lsn: Option<String>,
    pub(crate) updated_at: String,
}

pub(crate) fn snapshot_table_progress_from_parts(
    parts: SnapshotTableProgressParts,
) -> Result<SnapshotTableProgress> {
    let progress = SnapshotTableProgress {
        source_id: parts.source_id,
        dataset_id: parts.dataset_id,
        run_id: parts.run_id,
        relation: parts.relation,
        state: parts.state,
        copied_rows: parts.copied_rows,
        watermark_lsn: parts.watermark_lsn,
        updated_at: parts.updated_at,
    };
    validate_snapshot_table_progress_record(&progress)?;
    Ok(progress)
}

pub(crate) fn snapshot_table_progress_from_row(
    row: tokio_postgres::Row,
) -> Result<SnapshotTableProgress> {
    snapshot_table_progress_from_parts(SnapshotTableProgressParts {
        source_id: row.get(0),
        dataset_id: row.get(1),
        run_id: row.get(2),
        relation: row.get(3),
        state: row.get::<_, String>(4).parse()?,
        copied_rows: row.get(5),
        watermark_lsn: row.get(6),
        updated_at: row.get(7),
    })
}
