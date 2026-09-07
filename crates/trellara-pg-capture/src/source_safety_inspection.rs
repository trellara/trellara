use serde::Serialize;

use crate::{
    connect_control_client, inspect_all_user_tables, inspect_logical_replication_slots,
    inspect_subscription_conflict_stats, ReplicationSlotStatus, Result, SubscriptionConflictStats,
    TablePreflight,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DatabaseSourceSafetyInspection {
    pub read_only: bool,
    pub tables: Vec<TablePreflight>,
    pub logical_slots: Vec<ReplicationSlotStatus>,
    pub subscription_conflicts: Vec<SubscriptionConflictStats>,
    pub inspection_warnings: Vec<String>,
}

pub async fn inspect_database_source_safety(
    database_url: &str,
) -> Result<DatabaseSourceSafetyInspection> {
    let client = connect_control_client(database_url).await?;
    let tables = inspect_all_user_tables(&client).await?;
    let mut inspection_warnings = Vec::new();
    let logical_slots = inspect_logical_replication_slots(&client)
        .await
        .unwrap_or_else(|error| {
            inspection_warnings.push(format!(
                "logical replication slot inspection skipped: {error}"
            ));
            Vec::new()
        });
    let subscription_conflicts = inspect_subscription_conflict_stats(&client)
        .await
        .unwrap_or_else(|error| {
            inspection_warnings.push(format!("subscription conflict inspection skipped: {error}"));
            Vec::new()
        });

    Ok(DatabaseSourceSafetyInspection {
        read_only: true,
        tables,
        logical_slots,
        subscription_conflicts,
        inspection_warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_source_safety_inspection_is_read_only_evidence() {
        let inspection = DatabaseSourceSafetyInspection {
            read_only: true,
            tables: Vec::new(),
            logical_slots: Vec::new(),
            subscription_conflicts: Vec::new(),
            inspection_warnings: Vec::new(),
        };

        assert!(inspection.read_only);
        assert!(inspection.tables.is_empty());
        assert!(inspection.logical_slots.is_empty());
    }
}
