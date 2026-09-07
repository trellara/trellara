use tokio_postgres::Client;

use crate::{LogicalReplicationConflictCounts, Result, SubscriptionConflictStats};

pub(crate) const SUBSCRIPTION_STATS_VIEW_SQL: &str =
    "select to_regclass('pg_catalog.pg_stat_subscription_stats')::text as view_name";
pub(crate) const SUBSCRIPTION_CONFLICT_STATS_SQL: &str = r#"
            select ss.subid::text as subscription_id,
                   ss.subname::text as subscription_name,
                   coalesce(nullif(to_jsonb(ss)->>'apply_error_count', '')::bigint, 0) as apply_error_count,
                   coalesce(nullif(to_jsonb(ss)->>'sync_error_count', '')::bigint, 0) as sync_error_count,
                   coalesce(nullif(to_jsonb(ss)->>'confl_insert_exists', '')::bigint, 0) as confl_insert_exists,
                   coalesce(nullif(to_jsonb(ss)->>'confl_update_origin_differs', '')::bigint, 0) as confl_update_origin_differs,
                   coalesce(nullif(to_jsonb(ss)->>'confl_update_exists', '')::bigint, 0) as confl_update_exists,
                   coalesce(nullif(to_jsonb(ss)->>'confl_update_missing', '')::bigint, 0) as confl_update_missing,
                   coalesce(nullif(to_jsonb(ss)->>'confl_delete_origin_differs', '')::bigint, 0) as confl_delete_origin_differs,
                   coalesce(nullif(to_jsonb(ss)->>'confl_delete_missing', '')::bigint, 0) as confl_delete_missing,
                   coalesce(nullif(to_jsonb(ss)->>'confl_multiple_unique_conflicts', '')::bigint, 0) as confl_multiple_unique_conflicts,
                   to_jsonb(ss)->>'stats_reset' as stats_reset
              from pg_catalog.pg_stat_subscription_stats ss
             order by ss.subname
            "#;

pub(crate) async fn inspect_subscription_conflict_stats(
    client: &Client,
) -> Result<Vec<SubscriptionConflictStats>> {
    let view = client
        .query_one(SUBSCRIPTION_STATS_VIEW_SQL, &[])
        .await?
        .get::<_, Option<String>>("view_name");
    if view.is_none() {
        return Ok(Vec::new());
    }

    let rows = client.query(SUBSCRIPTION_CONFLICT_STATS_SQL, &[]).await?;

    Ok(rows
        .into_iter()
        .map(subscription_conflict_stats_from_row)
        .collect())
}

fn subscription_conflict_stats_from_row(row: tokio_postgres::Row) -> SubscriptionConflictStats {
    SubscriptionConflictStats {
        subscription_id: row.get("subscription_id"),
        subscription_name: row.get("subscription_name"),
        apply_error_count: row.get("apply_error_count"),
        sync_error_count: row.get("sync_error_count"),
        conflicts: LogicalReplicationConflictCounts {
            insert_exists: row.get("confl_insert_exists"),
            update_origin_differs: row.get("confl_update_origin_differs"),
            update_exists: row.get("confl_update_exists"),
            update_missing: row.get("confl_update_missing"),
            delete_origin_differs: row.get("confl_delete_origin_differs"),
            delete_missing: row.get("confl_delete_missing"),
            multiple_unique_conflicts: row.get("confl_multiple_unique_conflicts"),
        },
        stats_reset: row.get("stats_reset"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscription_conflict_queries_are_read_only() {
        assert!(SUBSCRIPTION_STATS_VIEW_SQL.contains("to_regclass"));
        assert!(SUBSCRIPTION_CONFLICT_STATS_SQL.contains("pg_stat_subscription_stats"));
        assert!(!SUBSCRIPTION_CONFLICT_STATS_SQL.contains("delete from"));
    }
}
