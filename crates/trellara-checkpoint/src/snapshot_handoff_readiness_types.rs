use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::SnapshotTableProgress;

pub struct SnapshotHandoffReadinessInput<'a> {
    pub source_id: &'a str,
    pub dataset_id: &'a str,
    pub run_id: &'a str,
    pub consistent_lsn: &'a str,
    pub relations: &'a [String],
    pub progress: &'a HashMap<String, SnapshotTableProgress>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SnapshotHandoffReadinessReport {
    pub ready: bool,
    pub relation_count: usize,
    pub copied_rows_at_boundary: i64,
    pub blockers: Vec<SnapshotHandoffReadinessBlocker>,
    pub recovery_actions: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SnapshotHandoffReadinessBlocker {
    pub code: String,
    pub relation: String,
    pub detail: String,
}
