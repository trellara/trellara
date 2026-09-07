use serde::{Deserialize, Serialize};
use trellara_protocol::RelationId;

use crate::{table_snapshot_checksum::table_snapshot_checksum, RowSnapshot};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TableSnapshot {
    pub relation: RelationId,
    pub watermark_lsn: String,
    pub rows: Vec<RowSnapshot>,
}

impl TableSnapshot {
    pub fn canonical(mut self) -> Self {
        self.rows
            .sort_by(|left, right| left.primary_key.cmp(&right.primary_key));
        self
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn checksum(&self) -> u64 {
        table_snapshot_checksum(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostgresSnapshotConfig {
    pub database_url: String,
    pub relation: RelationId,
    pub primary_key: String,
    pub excluded_columns: Vec<String>,
    pub row_filter: Option<String>,
    pub watermark_lsn: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostgresReseedConfig {
    pub source_database_url: String,
    pub target_database_url: String,
    pub relation: RelationId,
    pub primary_key: String,
    pub excluded_columns: Vec<String>,
    pub target_owned_columns: Vec<String>,
    pub row_filter: Option<String>,
    pub watermark_lsn: String,
    pub source_snapshot_name: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostgresRelationInspectionConfig {
    pub database_url: String,
    pub relation: RelationId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PostgresRelationInspection {
    pub relation: RelationId,
    pub exists: bool,
    pub columns: Vec<PostgresColumnInspection>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PostgresColumnInspection {
    pub ordinal_position: i32,
    pub name: String,
    pub type_name: String,
    pub nullable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PostgresReseedSummary {
    pub relation: RelationId,
    pub watermark_lsn: String,
    pub copied_rows: u64,
    pub copied_columns: Vec<String>,
}

impl PostgresSnapshotConfig {
    pub fn new(
        database_url: impl Into<String>,
        relation: RelationId,
        primary_key: impl Into<String>,
        watermark_lsn: impl Into<String>,
    ) -> Self {
        Self {
            database_url: database_url.into(),
            relation,
            primary_key: primary_key.into(),
            excluded_columns: Vec::new(),
            row_filter: None,
            watermark_lsn: watermark_lsn.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TableComparison {
    pub relation: RelationId,
    pub target_relation: RelationId,
    pub relation_match: bool,
    pub source_watermark_lsn: String,
    pub target_watermark_lsn: String,
    pub target_caught_up: bool,
    pub source_row_count: usize,
    pub target_row_count: usize,
    pub source_checksum: u64,
    pub target_checksum: u64,
    pub missing_in_target_count: usize,
    pub extra_in_target_count: usize,
    pub mismatched_row_count: usize,
    pub drift_sample_limit: usize,
    pub missing_in_target: Vec<String>,
    pub extra_in_target: Vec<String>,
    pub mismatched_rows: Vec<String>,
    pub evidence_sha256: String,
}

impl TableComparison {
    pub fn is_converged(&self) -> bool {
        self.relation_match
            && self.target_caught_up
            && self.source_row_count == self.target_row_count
            && self.source_checksum == self.target_checksum
            && self.missing_in_target_count == 0
            && self.extra_in_target_count == 0
            && self.mismatched_row_count == 0
    }
}
