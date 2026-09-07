use serde::Serialize;

use crate::ChecksumStatus;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct TableReseedSummary {
    pub(crate) relation: String,
    pub(crate) row_filter: Option<String>,
    pub(crate) watermark_lsn: String,
    pub(crate) copied_rows: u64,
    pub(crate) copied_columns: Vec<String>,
}

impl TableReseedSummary {
    pub(crate) fn from_summary(
        summary: trellara_verify::PostgresReseedSummary,
        row_filter: Option<String>,
    ) -> Self {
        Self {
            relation: summary.relation.display_name(),
            row_filter,
            watermark_lsn: summary.watermark_lsn,
            copied_rows: summary.copied_rows,
            copied_columns: summary.copied_columns,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct TableVerifySummary {
    pub(crate) relation: String,
    pub(crate) target_relation: String,
    pub(crate) relation_match: bool,
    pub(crate) row_filter: Option<String>,
    pub(crate) converged: bool,
    pub(crate) checksum_status: ChecksumStatus,
    pub(crate) source_row_count: usize,
    pub(crate) target_row_count: usize,
    pub(crate) source_checksum: u64,
    pub(crate) target_checksum: u64,
    pub(crate) missing_in_target_count: usize,
    pub(crate) extra_in_target_count: usize,
    pub(crate) mismatched_row_count: usize,
    pub(crate) drift_sample_limit: usize,
    pub(crate) missing_in_target: Vec<String>,
    pub(crate) extra_in_target: Vec<String>,
    pub(crate) mismatched_rows: Vec<String>,
    pub(crate) evidence_sha256: String,
    pub(crate) recommended_action: String,
}

impl TableVerifySummary {
    pub(crate) fn from_comparison(
        comparison: trellara_verify::TableComparison,
        row_filter: Option<String>,
    ) -> Self {
        let recommended_action = table_verify_recommended_action(&comparison);
        Self {
            relation: comparison.relation.display_name(),
            target_relation: comparison.target_relation.display_name(),
            relation_match: comparison.relation_match,
            row_filter,
            converged: comparison.is_converged(),
            checksum_status: table_checksum_status(&comparison),
            source_row_count: comparison.source_row_count,
            target_row_count: comparison.target_row_count,
            source_checksum: comparison.source_checksum,
            target_checksum: comparison.target_checksum,
            missing_in_target_count: comparison.missing_in_target_count,
            extra_in_target_count: comparison.extra_in_target_count,
            mismatched_row_count: comparison.mismatched_row_count,
            drift_sample_limit: comparison.drift_sample_limit,
            missing_in_target: comparison.missing_in_target,
            extra_in_target: comparison.extra_in_target,
            mismatched_rows: comparison.mismatched_rows,
            evidence_sha256: comparison.evidence_sha256,
            recommended_action,
        }
    }
}

fn table_checksum_status(comparison: &trellara_verify::TableComparison) -> ChecksumStatus {
    if !comparison.relation_match {
        return ChecksumStatus::Mismatch;
    }
    ChecksumStatus::from_checksums(
        comparison.source_checksum,
        comparison.target_checksum,
        comparison.source_row_count,
        comparison.target_row_count,
    )
}

pub(crate) fn table_verify_recommended_action(
    comparison: &trellara_verify::TableComparison,
) -> String {
    if comparison.is_converged() {
        return "no action; table converged at the verified watermark".to_string();
    }

    let relation = comparison.relation.display_name();
    let mut reasons = Vec::new();
    if !comparison.relation_match {
        reasons.push(format!(
            "target relation {} does not match source relation {relation}",
            comparison.target_relation.display_name()
        ));
    }
    if comparison.missing_in_target_count > 0 {
        reasons.push(format!(
            "{} source rows are missing in target",
            comparison.missing_in_target_count
        ));
    }
    if comparison.extra_in_target_count > 0 {
        reasons.push(format!(
            "{} target rows are not present in source",
            comparison.extra_in_target_count
        ));
    }
    if comparison.mismatched_row_count > 0 {
        reasons.push(format!(
            "{} rows have checksum mismatches",
            comparison.mismatched_row_count
        ));
    }
    if reasons.is_empty() {
        reasons.push(format!(
            "row counts or table checksums differ (source_rows={}, target_rows={})",
            comparison.source_row_count, comparison.target_row_count
        ));
    }

    format!(
        "repair {relation}: {}; run trellara reseed --config <config> --table {relation}, then rerun trellara verify --config <config> --table {relation}",
        reasons.join("; ")
    )
}
