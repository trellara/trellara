use trellara_pg_capture::SubscriptionConflictStats;

use crate::SourceSafetyFactor;

pub(crate) fn subscription_conflict_factors(
    stats: &[SubscriptionConflictStats],
) -> Vec<SourceSafetyFactor> {
    let mut factors = Vec::new();
    for stat in stats {
        let total_conflicts = stat.total_conflicts();
        if total_conflicts > 0 {
            factors.push(SourceSafetyFactor::critical(
                "logical_replication_conflicts",
                35,
                format!(
                    "subscription {} has {} logical replication conflict(s): insert_exists={}, update_origin_differs={}, update_exists={}, update_missing={}, delete_origin_differs={}, delete_missing={}, multiple_unique_conflicts={}",
                    stat.subscription_name,
                    total_conflicts,
                    stat.conflicts.insert_exists,
                    stat.conflicts.update_origin_differs,
                    stat.conflicts.update_exists,
                    stat.conflicts.update_missing,
                    stat.conflicts.delete_origin_differs,
                    stat.conflicts.delete_missing,
                    stat.conflicts.multiple_unique_conflicts
                ),
                "pause affected subscriptions, resolve subscriber-side divergence, then verify or reseed before trusting CDC output",
            ));
        } else if stat.apply_error_count > 0 || stat.sync_error_count > 0 {
            factors.push(SourceSafetyFactor::warning(
                "logical_replication_apply_errors",
                15,
                format!(
                    "subscription {} has apply_error_count={} and sync_error_count={}",
                    stat.subscription_name, stat.apply_error_count, stat.sync_error_count
                ),
                "inspect subscriber logs and run verification before treating replication as healthy",
            ));
        }
    }
    factors
}
