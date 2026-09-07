use std::collections::BTreeMap;

use crate::{
    comparison_digest::comparison_evidence_sha256, Result, RowSnapshot, TableComparison,
    TableSnapshot, VerifyError, DEFAULT_DRIFT_SAMPLE_LIMIT,
};

pub fn compare_snapshots(source: TableSnapshot, target: TableSnapshot) -> TableComparison {
    compare_snapshots_with_sample_limit(source, target, DEFAULT_DRIFT_SAMPLE_LIMIT)
}

pub fn compare_snapshots_with_sample_limit(
    source: TableSnapshot,
    target: TableSnapshot,
    drift_sample_limit: usize,
) -> TableComparison {
    let source = source.canonical();
    let target = target.canonical();
    let source_rows = row_map(&source.rows);
    let target_rows = row_map(&target.rows);

    let missing_in_target = source_rows
        .keys()
        .filter(|key| !target_rows.contains_key(*key))
        .cloned()
        .collect::<Vec<_>>();
    let extra_in_target = target_rows
        .keys()
        .filter(|key| !source_rows.contains_key(*key))
        .cloned()
        .collect::<Vec<_>>();
    let mismatched_rows = source_rows
        .iter()
        .filter_map(|(key, source_hash)| {
            target_rows
                .get(key)
                .filter(|target_hash| *target_hash != source_hash)
                .map(|_| key.clone())
        })
        .collect::<Vec<_>>();
    let missing_in_target_count = missing_in_target.len();
    let extra_in_target_count = extra_in_target.len();
    let mismatched_row_count = mismatched_rows.len();
    let target_caught_up =
        watermark_caught_up(&source.watermark_lsn, &target.watermark_lsn).unwrap_or(false);

    let mut comparison = TableComparison {
        relation: source.relation.clone(),
        target_relation: target.relation.clone(),
        relation_match: source.relation == target.relation,
        source_watermark_lsn: source.watermark_lsn.clone(),
        target_watermark_lsn: target.watermark_lsn.clone(),
        target_caught_up,
        source_row_count: source.row_count(),
        target_row_count: target.row_count(),
        source_checksum: source.checksum(),
        target_checksum: target.checksum(),
        missing_in_target_count,
        extra_in_target_count,
        mismatched_row_count,
        drift_sample_limit,
        missing_in_target: sample_keys(missing_in_target, drift_sample_limit),
        extra_in_target: sample_keys(extra_in_target, drift_sample_limit),
        mismatched_rows: sample_keys(mismatched_rows, drift_sample_limit),
        evidence_sha256: String::new(),
    };
    comparison.evidence_sha256 = comparison_evidence_sha256(&comparison);
    comparison
}

pub fn ensure_target_caught_up(source_lsn: &str, target_lsn: &str) -> Result<()> {
    if watermark_caught_up(source_lsn, target_lsn)? {
        Ok(())
    } else {
        Err(VerifyError::TargetBehind {
            source_lsn: source_lsn.to_string(),
            target_lsn: target_lsn.to_string(),
        })
    }
}

fn watermark_caught_up(source_lsn: &str, target_lsn: &str) -> Result<bool> {
    let source = parse_watermark_lsn("source", source_lsn)?;
    let target = parse_watermark_lsn("target", target_lsn)?;
    Ok(target >= source)
}

fn parse_watermark_lsn(field: &str, lsn: &str) -> Result<u64> {
    trellara_protocol::parse_lsn(lsn).map_err(|_| VerifyError::InvalidWatermarkLsn {
        field: field.to_string(),
        lsn: lsn.to_string(),
    })
}

pub fn parse_lsn(lsn: &str) -> u64 {
    let Some((high, low)) = lsn.split_once('/') else {
        return 0;
    };
    let high = u64::from_str_radix(high, 16).unwrap_or(0);
    let low = u64::from_str_radix(low, 16).unwrap_or(0);
    (high << 32) + low
}

fn row_map(rows: &[RowSnapshot]) -> BTreeMap<String, u64> {
    rows.iter()
        .map(|row| (row.primary_key.clone(), row.row_hash))
        .collect()
}

fn sample_keys(mut keys: Vec<String>, limit: usize) -> Vec<String> {
    keys.truncate(limit);
    keys
}
