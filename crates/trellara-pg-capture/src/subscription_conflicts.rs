use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SubscriptionConflictStats {
    pub subscription_id: String,
    pub subscription_name: String,
    pub apply_error_count: i64,
    pub sync_error_count: i64,
    pub conflicts: LogicalReplicationConflictCounts,
    pub stats_reset: Option<String>,
}

impl SubscriptionConflictStats {
    pub fn total_conflicts(&self) -> i64 {
        self.conflicts.total()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct LogicalReplicationConflictCounts {
    pub insert_exists: i64,
    pub update_origin_differs: i64,
    pub update_exists: i64,
    pub update_missing: i64,
    pub delete_origin_differs: i64,
    pub delete_missing: i64,
    pub multiple_unique_conflicts: i64,
}

impl LogicalReplicationConflictCounts {
    pub fn total(&self) -> i64 {
        self.insert_exists
            + self.update_origin_differs
            + self.update_exists
            + self.update_missing
            + self.delete_origin_differs
            + self.delete_missing
            + self.multiple_unique_conflicts
    }
}
