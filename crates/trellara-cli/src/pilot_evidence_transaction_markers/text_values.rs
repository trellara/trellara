use trellara_checkpoint::{lsn_shape_is_valid, parse_lsn};

use crate::pilot_evidence_identity_helpers::meaningful_str;

pub(super) fn text_key_value<'a>(lower: &'a str, key: &str) -> Option<&'a str> {
    lower
        .split_whitespace()
        .find_map(|token| token_value_for_key(token, key))
}

pub(super) fn line_lsn_field_valid(line: &str, key: &str) -> bool {
    lsn_value_for_key_in_line(line, key).is_some_and(lsn_valid)
}

pub(super) fn lsn_value_for_key_in_line<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.split_whitespace()
        .find_map(|token| lsn_value_for_key(token, key))
}

pub(super) fn value_for_key_in_line<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.split_whitespace()
        .find_map(|token| token_value_for_key(token, key))
}

fn lsn_value_for_key<'a>(token: &'a str, key: &str) -> Option<&'a str> {
    token_value_for_key(token, key).filter(|value| value.contains('/'))
}

fn token_value_for_key<'a>(token: &'a str, key: &str) -> Option<&'a str> {
    let value = token
        .strip_prefix(&format!("{key}="))
        .or_else(|| token.strip_prefix(&format!("{key}:")))?;
    let value = value.trim_matches(|ch: char| matches!(ch, ',' | ';' | '"' | '\'' | '{' | '}'));
    meaningful_str(value).then_some(value)
}

pub(super) fn lsn_valid(value: &str) -> bool {
    lsn_shape_is_valid(value) && parse_lsn(value) > 0
}
