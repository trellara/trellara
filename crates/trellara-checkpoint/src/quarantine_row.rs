use crate::{quarantine_validation::validate_quarantine_record, ApplyQuarantine, Result};

pub(crate) struct QuarantineRowParts {
    pub(crate) source_id: String,
    pub(crate) database_id: String,
    pub(crate) dataset_id: String,
    pub(crate) transaction_id: String,
    pub(crate) commit_lsn: String,
    pub(crate) reason: String,
    pub(crate) detail: String,
    pub(crate) attempt_count: i64,
    pub(crate) last_seen_at: String,
}

pub(crate) fn quarantine_from_parts(parts: QuarantineRowParts) -> Result<ApplyQuarantine> {
    let record = ApplyQuarantine {
        source_id: parts.source_id,
        database_id: parts.database_id,
        dataset_id: parts.dataset_id,
        transaction_id: parts.transaction_id,
        commit_lsn: parts.commit_lsn,
        reason: parts.reason,
        detail: parts.detail,
        attempt_count: parts.attempt_count,
        last_seen_at: parts.last_seen_at,
    };
    validate_quarantine_record(&record)?;
    Ok(record)
}

pub(crate) fn quarantine_from_row(row: tokio_postgres::Row) -> Result<ApplyQuarantine> {
    quarantine_from_parts(QuarantineRowParts {
        source_id: row.get(0),
        database_id: row.get(1),
        dataset_id: row.get(2),
        transaction_id: row.get(3),
        commit_lsn: row.get(4),
        reason: row.get(5),
        detail: row.get(6),
        attempt_count: row.get(7),
        last_seen_at: row.get(8),
    })
}
