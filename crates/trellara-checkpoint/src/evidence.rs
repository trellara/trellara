use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApplyQuarantine {
    pub source_id: String,
    pub database_id: String,
    pub dataset_id: String,
    pub transaction_id: String,
    pub commit_lsn: String,
    pub reason: String,
    pub detail: String,
    pub attempt_count: i64,
    pub last_seen_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReseedEvent {
    pub source_id: String,
    pub dataset_id: String,
    pub watermark_lsn: String,
    pub table_count: i64,
    pub copied_rows: i64,
    pub completed_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SnapshotHandoffEvent {
    pub source_id: String,
    pub dataset_id: String,
    pub relation: String,
    pub watermark_lsn: String,
    pub copied_rows: i64,
    pub completed_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ValidationEvent {
    pub source_id: String,
    pub dataset_id: String,
    pub source_watermark_lsn: String,
    pub target_watermark_lsn: String,
    pub converged: bool,
    pub table_count: i64,
    pub drift_count: i64,
    pub drift_relations: Vec<String>,
    pub evidence_sha256: Option<String>,
    pub completed_at: String,
}
