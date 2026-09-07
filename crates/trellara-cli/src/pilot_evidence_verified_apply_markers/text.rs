use trellara_checkpoint::parse_lsn;

use super::lsn_valid;
use crate::pilot_evidence_identity_helpers::meaningful_str;

pub(super) fn verified_apply_identity_present(contents: &str) -> bool {
    let lower = contents.to_ascii_lowercase();
    text_key_value(&lower, "source_id").is_some() && text_key_value(&lower, "dataset_id").is_some()
}

pub(super) fn verified_apply_complete(contents: &str) -> bool {
    let lower = contents.to_ascii_lowercase();
    (lower.contains("converged=true") || lower.contains("converged: true"))
        && (lower.contains("checksum_status=match") || lower.contains("checksum_status: match"))
        && text_watermarks_match(&lower)
        && text_table_count_positive(&lower)
        && text_table_relation_present(&lower)
        && text_relation_identity_matches(&lower)
        && !lower.contains("relation_match=false")
        && !lower.contains("relation_match: false")
        && !lower.contains("checksum_status=mismatch")
        && !lower.contains("checksum_status: mismatch")
        && !lower.contains("converged=false")
        && !lower.contains("converged: false")
}

pub(super) fn verified_apply_target_relation_identity(contents: &str) -> bool {
    let lower = contents.to_ascii_lowercase();
    text_key_value(&lower, "relation_match").is_some_and(|value| value == "true")
        && text_key_value(&lower, "target_relation")
            .zip(text_key_value(&lower, "relation"))
            .is_some_and(|(target_relation, relation)| target_relation == relation)
}

fn text_watermarks_match(lower: &str) -> bool {
    let Some(source) = text_key_value(lower, "source_watermark_lsn") else {
        return false;
    };
    let Some(target) = text_key_value(lower, "target_watermark_lsn") else {
        return false;
    };
    lsn_valid(source) && lsn_valid(target) && parse_lsn(source) == parse_lsn(target)
}

fn text_key_value<'a>(lower: &'a str, key: &str) -> Option<&'a str> {
    lower
        .split_whitespace()
        .filter_map(|part| part.strip_prefix(&format!("{key}=")))
        .find_map(|value| {
            let value = value.trim_matches(|ch: char| ch == ',' || ch == ';');
            meaningful_str(value).then_some(value)
        })
}

fn text_table_count_positive(lower: &str) -> bool {
    lower
        .split_whitespace()
        .filter_map(|part| part.strip_prefix("table_count="))
        .filter_map(|value| {
            value
                .trim_matches(|ch: char| ch == ',' || ch == ';')
                .parse::<usize>()
                .ok()
        })
        .any(|count| count > 0)
}

fn text_table_relation_present(lower: &str) -> bool {
    text_key_value(lower, "relation")
        .or_else(|| text_key_value(lower, "table"))
        .is_some_and(|relation| relation.contains('.') && !relation.ends_with('.'))
}

fn text_relation_identity_matches(lower: &str) -> bool {
    let Some(target_relation) = text_key_value(lower, "target_relation") else {
        return true;
    };
    text_key_value(lower, "relation").is_some_and(|relation| relation == target_relation)
}
