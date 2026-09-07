use std::collections::BTreeSet;

use serde_json::Value;

use super::sinks::{required_sink_set, sink_is_supported};

mod command_lsn;
mod command_tokens;
mod partition_command;

use command_tokens::{clean_arg, command_arg, command_args};

pub(super) fn is_executable(item: &Value) -> bool {
    let Some(command) = item.as_str().map(str::to_ascii_lowercase) else {
        return false;
    };
    is_executable_text(&command)
}

pub(super) fn collection_is_valid(items: &[Value], proof: &Value) -> bool {
    let commands = items
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    let barrier_lsn = proof.get("barrier_lsn").and_then(Value::as_str);

    commands_have_unique_sinks(&commands)
        && commands.iter().all(|command| {
            is_executable_text(command)
                && command_lsn::ack_lsn_reaches_barrier(
                    command_arg(command, "--ack-lsn"),
                    barrier_lsn,
                )
                && matches_required_arg(command, proof, "barrier_id", "--barrier-id")
                && matches_required_arg(command, proof, "schema_version", "--schema-version")
                && sink_specific_args_are_valid(command)
        })
        && covers_required_sinks(&commands, proof)
}

fn is_executable_text(command: &str) -> bool {
    command.contains("trellara schema ddl-barrier ack")
        && clean_arg(command, "--config")
        && clean_arg(command, "--sink")
}

fn covers_required_sinks(commands: &[String], proof: &Value) -> bool {
    let Some(required_sinks) = required_sink_set(proof) else {
        return false;
    };
    if required_sinks.is_empty() {
        return false;
    }

    let command_sinks: BTreeSet<String> = commands
        .iter()
        .filter_map(|command| command_arg(command, "--sink"))
        .map(|sink| sink.trim().to_string())
        .filter(|sink| !sink.is_empty())
        .collect();

    command_sinks == required_sinks
}

fn commands_have_unique_sinks(commands: &[String]) -> bool {
    let mut sinks = BTreeSet::new();
    commands
        .iter()
        .filter_map(|command| command_arg(command, "--sink"))
        .map(|sink| sink.trim().to_string())
        .all(|sink| !sink.is_empty() && sinks.insert(sink))
}

fn sink_specific_args_are_valid(command: &str) -> bool {
    match command_arg(command, "--sink").as_deref() {
        Some("target_postgres") => target_postgres_args_are_valid(command),
        Some("raw_cdc_lake") => raw_cdc_lake_args_are_valid(command),
        Some("spark_derived_views") => spark_derived_views_args_are_valid(command),
        Some("partition_visibility") => partition_command::args_are_valid(
            command_arg(command, "--barrier-lsn"),
            command_arg(command, "--expected-partition-count"),
            command_args(command, "--partition-durable-lsn"),
            command_args(command, "--partition-applied-lsn"),
        ),
        Some(sink) => sink_is_supported(sink),
        None => false,
    }
}

fn target_postgres_args_are_valid(command: &str) -> bool {
    let statement_sha256s = command_args(command, "--statement-sha256");
    command_arg(command, "--plan-sha256").is_some_and(|sha| sha256_is_valid(&sha))
        && !statement_sha256s.is_empty()
        && statement_sha256s.iter().all(|sha| sha256_is_valid(sha))
}

fn raw_cdc_lake_args_are_valid(command: &str) -> bool {
    clean_arg(command, "--epoch-id")
        && clean_arg(command, "--metadata-table")
        && clean_arg(command, "--partition-metadata-table")
        && command_arg(command, "--manifest-digest").is_some_and(|sha| sha256_is_valid(&sha))
}

fn spark_derived_views_args_are_valid(command: &str) -> bool {
    command_arg(command, "--template-digest").is_some_and(|sha| sha256_is_valid(&sha))
        && clean_arg(command, "--accepted-by")
        && command_arg(command, "--view-count")
            .and_then(|count| count.parse::<u32>().ok())
            .is_some_and(|count| count > 0)
}

fn matches_required_arg(
    command: &str,
    proof: &Value,
    proof_field: &str,
    command_flag: &str,
) -> bool {
    proof
        .get(proof_field)
        .and_then(Value::as_str)
        .is_some_and(|expected| {
            let expected = expected.to_ascii_lowercase();
            !expected.trim().is_empty()
                && command_arg(command, command_flag).is_some_and(|actual| actual == expected)
        })
}

fn sha256_is_valid(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}
