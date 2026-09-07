use super::text_values::value_for_key_in_line;

pub(super) fn local_publish_ack_proof_present(lower: &str, compact: &str) -> bool {
    (lower.contains(
        "last_publish_ack_proof=local_publish_ack_is_indexed_replayable_and_untorn_before_source_ack",
    ) || compact.contains(
        "\"contract\":\"local_publish_ack_is_indexed_replayable_and_untorn_before_source_ack\"",
    ))
        && (lower.contains("last_publish_ack_indexed=true")
            || compact.contains("\"indexed\":true"))
        && (lower.contains("last_publish_ack_replayable=true")
            || compact.contains("\"replayable\":true"))
        && (lower.contains("last_publish_ack_crash_safe_ack=true")
            || compact.contains("\"crash_safe_ack\":true"))
        && (lower.contains("last_publish_ack_torn_tail_bytes=0")
            || compact.contains("\"torn_tail_bytes\":0"))
}

pub(super) fn local_source_ack_durability_proof_present(lower: &str, compact: &str) -> bool {
    (lower.contains(
        "source_ack_durability_proof=every_local_publish_ack_is_indexed_replayable_untorn_and_destination_matched_before_source_ack",
    ) || compact.contains(
        "\"contract\":\"every_local_publish_ack_is_indexed_replayable_untorn_and_destination_matched_before_source_ack\"",
    ))
        && (lower.contains("source_ack_all_publish_acks_proven=true")
            || compact.contains("\"all_publish_acks_proven\":true"))
        && (lower.contains("source_ack_crash_safe_ack=true")
            || compact.contains("\"crash_safe_ack\":true"))
        && matching_positive_source_ack_counts(lower, compact)
}

fn matching_positive_source_ack_counts(lower: &str, compact: &str) -> bool {
    let expected = source_ack_expected_publish_messages(lower, compact);
    let durable = source_ack_count_value(lower, "source_ack_durable_publish_acks")
        .or_else(|| json_count_value(compact, "durable_publish_acks"));
    let proofed = source_ack_count_value(lower, "source_ack_proofed_ack_count")
        .or_else(|| json_count_value(compact, "proofed_ack_count"));
    matches!(
        (expected, durable, proofed),
        (Some(expected), Some(durable), Some(proofed))
            if expected > 0 && expected == durable && durable == proofed
    )
}

pub(super) fn source_ack_expected_publish_messages(lower: &str, compact: &str) -> Option<usize> {
    source_ack_count_value(lower, "source_ack_expected_publish_messages")
        .or_else(|| json_count_value(compact, "expected_publish_messages"))
}

fn source_ack_count_value(lower: &str, key: &str) -> Option<usize> {
    lower
        .lines()
        .find_map(|line| value_for_key_in_line(line, key).and_then(|value| value.parse().ok()))
}

fn json_count_value(compact: &str, key: &str) -> Option<usize> {
    compact
        .split(&format!("\"{key}\":"))
        .nth(1)
        .and_then(|tail| tail.split(|ch: char| !ch.is_ascii_digit()).next())
        .and_then(|digits| digits.parse::<usize>().ok())
}
