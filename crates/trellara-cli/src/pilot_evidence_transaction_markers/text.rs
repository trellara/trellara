use trellara_checkpoint::parse_lsn;

use super::text_values::{
    line_lsn_field_valid, lsn_valid, lsn_value_for_key_in_line, text_key_value,
    value_for_key_in_line,
};

pub(super) fn source_dataset_identity_present(lower: &str) -> bool {
    text_key_value(lower, "source_id").is_some() && text_key_value(lower, "dataset_id").is_some()
}

pub(super) fn transaction_boundary_verified(lower: &str, compact: &str) -> bool {
    (lower.contains("transaction_boundary: verified")
        || lower.contains("[verified] transaction_boundary")
        || lower.contains("transaction-boundary proof verified")
        || compact.contains("\"transaction_boundary\":{\"status\":\"verified\"")
        || compact.contains("\"transaction_boundary_status\":\"verified\""))
        && !lower.contains("transaction_boundary: at_risk")
        && !lower.contains("transaction_boundary: blocked")
        && !lower.contains("transaction_boundary: pending")
}

pub(super) fn checkpoint_evidence(lower: &str, checkpoint: &str) -> bool {
    lower
        .lines()
        .filter(|line| line.contains(checkpoint))
        .any(|line| checkpoint_line_has_required_lsns(lower, line, checkpoint))
}

pub(super) fn source_ack_lsn_evidence(lower: &str) -> bool {
    lower
        .lines()
        .filter(|line| line.contains("source_ack_lsn"))
        .filter_map(|line| {
            source_ack_line_can_be_evidence(line)
                .then(|| lsn_value_for_key_in_line(line, "source_ack_lsn"))
                .flatten()
        })
        .any(|source_ack_lsn| {
            lsn_valid(source_ack_lsn) && source_ack_covers_durable_checkpoint(lower, source_ack_lsn)
        })
}

pub(super) fn source_ack_after_durable_publish(lower: &str, compact: &str) -> bool {
    (lower.contains("every trellara publish ack is durable")
        || lower.contains("source acknowledgement advances after durable")
        || compact.contains("\"source_ack_after_durable_publish\":true")
        || lower.contains("source_ack_after_durable_publish=true"))
        && super::text_ack::local_source_ack_durability_proof_present(lower, compact)
        && super::text_ack::local_publish_ack_proof_present(lower, compact)
        && !lower.contains("source_ack_after_durable_publish=false")
        && !compact.contains("\"source_ack_after_durable_publish\":false")
}

pub(super) fn source_ack_publish_destinations(lower: &str, compact: &str) -> bool {
    let Some(destination_count) = destination_count(lower, compact) else {
        return false;
    };

    (lower.contains("source_ack_publish_destinations_match=true")
        || compact.contains("\"source_ack_publish_destinations_match\":true"))
        && !lower.contains("source_ack_publish_destinations_match=false")
        && !compact.contains("\"source_ack_publish_destinations_match\":false")
        && destination_count > 0
        && source_ack_destination_count_matches_proof(lower, compact, destination_count)
}

pub(super) fn parallel_replay_contract(lower: &str) -> bool {
    lower.contains("parallel_replay_contract")
        && (lower.contains("parallel replay is disabled")
            || (lower.contains("dml-only") && lower.contains("ddl barrier")))
}

fn destination_count(lower: &str, compact: &str) -> Option<usize> {
    lower
        .lines()
        .find_map(|line| {
            value_for_key_in_line(line, "source_ack_publish_destination_count")
                .and_then(|value| value.parse::<usize>().ok())
        })
        .or_else(|| {
            compact
                .split("\"source_ack_publish_destination_count\":")
                .nth(1)
                .and_then(|tail| tail.split(|ch: char| !ch.is_ascii_digit()).next())
                .and_then(|digits| digits.parse::<usize>().ok())
        })
}

fn source_ack_destination_count_matches_proof(
    lower: &str,
    compact: &str,
    destination_count: usize,
) -> bool {
    super::text_ack::source_ack_expected_publish_messages(lower, compact)
        .is_none_or(|expected| expected == destination_count)
}

fn checkpoint_line_has_required_lsns(lower: &str, line: &str, checkpoint: &str) -> bool {
    if line.contains(" missing") || line.contains("=missing") {
        return false;
    }

    match checkpoint {
        "source_checkpoint" => {
            line_lsn_field_valid(line, "last_seen_lsn")
                && line_lsn_field_valid(line, "last_durable_lsn")
        }
        "target_checkpoint" => {
            line_lsn_field_valid(line, "last_durable_lsn")
                && line_lsn_field_valid(line, "last_applied_lsn")
                && target_applied_covers_required_checkpoint(lower, line)
        }
        _ => false,
    }
}

fn source_ack_line_can_be_evidence(line: &str) -> bool {
    !line.contains("source_ack_lsn evidence")
        && !line.contains("source_ack_lsn=none")
        && !line.contains("source_ack_lsn=null")
        && !line.contains("source_ack_lsn=missing")
}

fn source_ack_covers_durable_checkpoint(lower: &str, source_ack_lsn: &str) -> bool {
    let Some(durable_lsn) = source_checkpoint_lsn(lower, "last_durable_lsn") else {
        return false;
    };
    parse_lsn(source_ack_lsn) >= parse_lsn(durable_lsn)
}

fn source_checkpoint_lsn<'a>(lower: &'a str, key: &str) -> Option<&'a str> {
    lower
        .lines()
        .filter(|line| line.contains("source_checkpoint"))
        .find_map(|line| lsn_value_for_key_in_line(line, key).filter(|value| lsn_valid(value)))
}

fn target_applied_covers_required_checkpoint(lower: &str, line: &str) -> bool {
    let Some(durable_lsn) =
        lsn_value_for_key_in_line(line, "last_durable_lsn").filter(|value| lsn_valid(value))
    else {
        return false;
    };
    let Some(applied_lsn) =
        lsn_value_for_key_in_line(line, "last_applied_lsn").filter(|value| lsn_valid(value))
    else {
        return false;
    };
    let required_lsn = source_checkpoint_lsn(lower, "last_durable_lsn")
        .map(parse_lsn)
        .unwrap_or_else(|| parse_lsn(durable_lsn))
        .max(parse_lsn(durable_lsn));
    parse_lsn(applied_lsn) >= required_lsn
}
