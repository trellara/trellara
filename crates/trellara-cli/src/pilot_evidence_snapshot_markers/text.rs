use trellara_checkpoint::parse_lsn;

use super::lsn_present;
use crate::pilot_evidence_identity_helpers::meaningful_str;

pub(super) fn source_dataset_identity_present(contents: &str) -> bool {
    let lower = contents.to_ascii_lowercase();
    text_identity_value_present(&lower, "source_id")
        && text_identity_value_present(&lower, "dataset_id")
}

pub(super) fn snapshot_handoff_complete(contents: &str) -> bool {
    let lower = contents.to_ascii_lowercase();
    (lower.contains("state=stream_handoff_ready")
        || lower.contains("state: stream_handoff_ready")
        || lower.contains("\"state\":\"stream_handoff_ready\"")
        || lower.contains("\"state\": \"stream_handoff_ready\""))
        && lower.contains("copy_complete")
        && text_lsn_value_present(&lower, "consistent_lsn")
        && text_watermark_matches_consistent_lsn(&lower)
        && text_selected_table_coverage(&lower)
        && text_table_count_positive(&lower)
}

fn text_lsn_value_present(lower: &str, key: &str) -> bool {
    lower
        .split_whitespace()
        .filter_map(|part| {
            part.strip_prefix(&format!("{key}="))
                .or_else(|| part.strip_prefix(&format!("{key}:")))
        })
        .any(|value| lsn_present(value.trim_matches(|ch: char| ch == ',' || ch == ';')))
}

fn text_watermark_matches_consistent_lsn(lower: &str) -> bool {
    let Some(consistent_lsn) = text_lsn_value(lower, "consistent_lsn") else {
        return false;
    };
    text_lsn_value(lower, "watermark_lsn")
        .or_else(|| text_lsn_value(lower, "handoff_watermark_lsn"))
        .or_else(|| text_handoff_watermark_value(lower))
        .is_some_and(|watermark_lsn| parse_lsn(watermark_lsn) == parse_lsn(consistent_lsn))
}

fn text_lsn_value<'a>(lower: &'a str, key: &str) -> Option<&'a str> {
    lower
        .split_whitespace()
        .filter_map(|part| {
            part.strip_prefix(&format!("{key}="))
                .or_else(|| part.strip_prefix(&format!("{key}:")))
        })
        .find_map(|value| {
            let value = value.trim_matches(|ch: char| ch == ',' || ch == ';');
            lsn_present(value).then_some(value)
        })
}

fn text_handoff_watermark_value(lower: &str) -> Option<&str> {
    lower.lines().find_map(|line| {
        line.strip_prefix("handoff watermark:")
            .map(str::trim)
            .filter(|value| lsn_present(value))
    })
}

fn text_table_count_positive(lower: &str) -> bool {
    text_usize_value(lower, "table_count").is_some_and(|count| count > 0)
}

pub(super) fn text_selected_table_coverage(contents: &str) -> bool {
    let lower = contents.to_ascii_lowercase();
    let Some(selected_table_count) = text_usize_value(&lower, "selected_table_count") else {
        return false;
    };
    let Some(table_count) = text_usize_value(&lower, "table_count") else {
        return false;
    };

    selected_table_count > 0 && selected_table_count == table_count
}

fn text_usize_value(lower: &str, key: &str) -> Option<usize> {
    lower
        .split_whitespace()
        .filter_map(|part| {
            part.strip_prefix(&format!("{key}="))
                .or_else(|| part.strip_prefix(&format!("{key}:")))
        })
        .find_map(|value| {
            value
                .trim_matches(|ch: char| ch == ',' || ch == ';')
                .parse::<usize>()
                .ok()
        })
}

fn text_identity_value_present(lower: &str, key: &str) -> bool {
    lower
        .split_whitespace()
        .filter_map(|part| part.strip_prefix(&format!("{key}=")))
        .any(|value| {
            let trimmed = value.trim_matches(|ch: char| ch == ',' || ch == ';');
            meaningful_str(trimmed)
        })
}
