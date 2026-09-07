use crate::{
    render_html::render_check_html, render_text::render_check_text, CheckOutputFormat,
    CheckSummary, Result,
};

pub fn render_check_summary(summary: &CheckSummary, format: CheckOutputFormat) -> Result<String> {
    match format {
        CheckOutputFormat::Html => Ok(render_check_html(summary)),
        CheckOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        CheckOutputFormat::Text => Ok(render_check_text(summary)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CheckFactor, CheckGrade, CheckSeverity, CheckStatus};
    use trellara_pg_capture::{ReplicationSlotStatus, WalHeadroomProjection};

    #[test]
    fn html_report_keeps_shareable_sql_and_headroom_contract() {
        let html =
            render_check_summary(&sample_summary(), CheckOutputFormat::Html).expect("html report");

        assert!(html.contains("Trellara check report"));
        assert!(html.contains("source_wal_retention_risk"));
        assert!(html.contains("~14 hours"));
        assert!(html.contains("Exact Read-Only Queries"));
        assert!(html.contains("logical_replication_slots"));
    }

    #[test]
    fn text_report_names_the_standalone_diagnostic() {
        let text =
            render_check_summary(&sample_summary(), CheckOutputFormat::Text).expect("text report");

        assert!(text.contains("Trellara check"));
        assert!(text.contains("logical_slots: 1 checked, 1 at_risk"));
        assert!(text.contains("exact_read_only_queries"));
    }

    fn sample_summary() -> CheckSummary {
        CheckSummary {
            tool: "trellara-check".to_string(),
            database: "postgresql://<redacted>@db.example.com/app".to_string(),
            read_only: true,
            managed_postgres_ready: "normal PostgreSQL TLS connection".to_string(),
            score: 80,
            grade: CheckGrade::B,
            status: CheckStatus::Degraded,
            table_count: 3,
            unsafe_table_count: 0,
            logical_slot_count: 1,
            at_risk_slot_count: 1,
            subscription_conflict_count: 0,
            inspection_warnings: Vec::new(),
            unsafe_tables: Vec::new(),
            logical_slots: vec![slot()],
            subscription_conflicts: Vec::new(),
            findings: vec![CheckFactor {
                code: "source_wal_retention_risk".to_string(),
                severity: CheckSeverity::Warning,
                points_lost: 20,
                evidence: "source slot slot-a has ~14 hours of safe WAL headroom".to_string(),
                recommendation: "move CDC pressure off the primary".to_string(),
            }],
            recommended_actions: vec!["move CDC pressure off the primary".to_string()],
        }
    }

    fn slot() -> ReplicationSlotStatus {
        ReplicationSlotStatus {
            slot_name: "slot-a".to_string(),
            exists: true,
            plugin: Some("pgoutput".to_string()),
            expected_plugin: "pgoutput".to_string(),
            active: Some(false),
            failover: None,
            synced: None,
            inactive_since: None,
            idle_replication_slot_timeout: None,
            restart_lsn: Some("0/16B6B00".to_string()),
            confirmed_flush_lsn: Some("0/16B6B00".to_string()),
            retained_wal_bytes: Some(1024),
            wal_status: Some("reserved".to_string()),
            safe_wal_size_bytes: Some(50_400_000),
            invalidation_reason: None,
            wal_headroom: Some(WalHeadroomProjection {
                source: "pg_current_wal_lsn_sample".to_string(),
                safe_wal_size_bytes: 50_400_000,
                wal_bytes_per_second: 1_000,
                sample_ms: 2_000,
                headroom_seconds: Some(50_400),
            }),
            transaction_id_wraparound: None,
            xmin_horizon: None,
            issues: Vec::new(),
        }
    }
}
