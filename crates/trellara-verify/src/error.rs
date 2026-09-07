use thiserror::Error;

#[derive(Debug, Error)]
pub enum VerifyError {
    #[error("source watermark {source_lsn} is ahead of target watermark {target_lsn}")]
    TargetBehind {
        source_lsn: String,
        target_lsn: String,
    },
    #[error("{field} watermark {lsn} is not a valid PostgreSQL LSN")]
    InvalidWatermarkLsn { field: String, lsn: String },
    #[error("postgres connection failed: {0}")]
    Postgres(#[from] tokio_postgres::Error),
    #[error("snapshot row for {relation} did not contain a JSON object")]
    RowNotObject { relation: String },
    #[error("failed to parse snapshot row JSON for {relation}: {source}")]
    ParseRowJson {
        relation: String,
        source: serde_json::Error,
    },
    #[error("reseed table {relation} has no copyable columns")]
    NoCopyableColumns { relation: String },
    #[error("reseed row count {row_count} exceeds supported u64 range")]
    ReseedRowCountOverflow { row_count: usize },
}
