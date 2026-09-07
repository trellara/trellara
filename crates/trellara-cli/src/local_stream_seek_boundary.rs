use std::path::{Path, PathBuf};

use trellara_stream_local::read_local_message_at;

use crate::{
    local_message_header, locate_configured_local_stream_transaction, CliError,
    LocalStreamLocateArgs, LocalStreamLocateBoundarySummary, Result, TrellaraConfig,
};

pub(crate) struct LocalSeekBoundaryContext {
    pub(crate) boundary: LocalStreamLocateBoundarySummary,
    pub(crate) seek_commands: Vec<String>,
}

pub(crate) fn local_seek_boundary_context(
    config: &TrellaraConfig,
    path: &Path,
    topic: &str,
    next_offset: i64,
) -> Result<Option<LocalSeekBoundaryContext>> {
    let Some(message) = read_local_message_at(path, topic, next_offset)? else {
        return Ok(None);
    };
    let Some(transaction_id) = local_message_header(&message, "trellara.transaction_id") else {
        return Ok(None);
    };
    let commit_lsn = local_message_header(&message, "trellara.commit_lsn").map(str::to_string);
    let locate = locate_configured_local_stream_transaction(
        config,
        &LocalStreamLocateArgs {
            config: PathBuf::new(),
            transaction_id: transaction_id.to_string(),
            commit_lsn,
            topic: None,
        },
    )?;

    Ok(Some(LocalSeekBoundaryContext {
        boundary: locate.boundary,
        seek_commands: locate.next_commands,
    }))
}

pub(crate) fn verify_seek_expected_boundary(
    path: &Path,
    topic: &str,
    next_offset: i64,
    transaction_id: Option<&str>,
    commit_lsn: Option<&str>,
) -> Result<()> {
    if transaction_id.is_none() && commit_lsn.is_none() {
        return Ok(());
    }
    let Some(message) = read_local_message_at(path, topic, next_offset)? else {
        return Err(CliError::InvalidConfig(format!(
            "stream.seek expected boundary at topic {topic} offset {next_offset}, but no local message exists"
        )));
    };
    if let Some(expected) = transaction_id {
        let actual = local_message_header(&message, "trellara.transaction_id");
        if actual != Some(expected) {
            return Err(CliError::InvalidConfig(format!(
                "stream.seek transaction_id mismatch at topic {topic} offset {next_offset}: expected {expected}, found {}",
                actual.unwrap_or("<missing>")
            )));
        }
    }
    if let Some(expected) = commit_lsn {
        let actual = local_message_header(&message, "trellara.commit_lsn");
        if actual != Some(expected) {
            return Err(CliError::InvalidConfig(format!(
                "stream.seek commit_lsn mismatch at topic {topic} offset {next_offset}: expected {expected}, found {}",
                actual.unwrap_or("<missing>")
            )));
        }
    }
    Ok(())
}

pub(crate) fn seek_boundary_warnings(boundary: &LocalStreamLocateBoundarySummary) -> Vec<String> {
    if boundary.complete {
        return Vec::new();
    }

    let mut warnings = vec![format!("local boundary status is {}", boundary.status)];
    if !boundary.missing_message_kinds.is_empty() {
        warnings.push(format!(
            "missing message kinds: {}",
            boundary.missing_message_kinds.join(", ")
        ));
    }
    if !boundary.missing_partition_ids.is_empty() {
        warnings.push(format!(
            "missing partition ids: {}",
            boundary
                .missing_partition_ids
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !boundary.metadata_conflicts.is_empty() {
        warnings.push(format!(
            "metadata conflicts: {}",
            boundary.metadata_conflicts.join("; ")
        ));
    }
    warnings
}

pub(crate) fn seek_replay_warnings(
    movement: &str,
    transaction_id: Option<&str>,
    commit_lsn: Option<&str>,
    boundary: Option<&LocalStreamLocateBoundarySummary>,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if movement == "rewind_replay" {
        if transaction_id.is_none() {
            warnings.push("transaction_id is required for replay-safe local rewind".to_string());
        }
        if commit_lsn.is_none() {
            warnings.push("commit_lsn is required for replay-safe exact local rewind".to_string());
        }
    }
    match boundary {
        Some(boundary) if !boundary.complete => {
            warnings.push(format!("local boundary status is {}", boundary.status))
        }
        None if movement == "rewind_replay" => warnings
            .push("no local transaction boundary evidence found at rewind offset".to_string()),
        _ => {}
    }
    warnings
}
