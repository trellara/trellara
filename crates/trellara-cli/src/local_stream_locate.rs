#[cfg(feature = "local-stream")]
use std::collections::BTreeMap;

use trellara_checkpoint::{lsn_shape_is_valid, parse_lsn};
#[cfg(feature = "local-stream")]
use trellara_stream_local::{inspect_local_stream, read_local_message_at};

#[cfg(feature = "local-stream")]
use crate::{
    local_message_header, local_message_matches_boundary, CliError, LocalStreamLocateArgs,
    LocalStreamLocateBoundarySummary, LocalStreamLocateMatch, LocalStreamLocateSummary, Result,
    StreamConfig, TrellaraConfig,
};
#[cfg(not(feature = "local-stream"))]
use crate::{CliError, LocalStreamLocateArgs, LocalStreamLocateSummary, Result, TrellaraConfig};

#[cfg(feature = "local-stream")]
pub(crate) fn locate_configured_local_stream_transaction(
    config: &TrellaraConfig,
    args: &LocalStreamLocateArgs,
) -> Result<LocalStreamLocateSummary> {
    validate_local_stream_locate_boundary(&args.transaction_id, args.commit_lsn.as_deref())?;
    let path = match &config.stream {
        StreamConfig::Local { path, .. } => path,
        StreamConfig::Kafka { .. } => {
            return Err(CliError::InvalidConfig(
                "stream.kind must be local for local stream transaction lookup".to_string(),
            ));
        }
    };
    let configured_topics = config.local_stream_topics()?;
    let topics_scanned = if let Some(topic) = &args.topic {
        if !configured_topics
            .iter()
            .any(|configured| configured == topic)
        {
            return Err(CliError::InvalidConfig(format!(
                "topic {topic} is not configured for this flow; configured topics: {}",
                configured_topics.join(", ")
            )));
        }
        vec![topic.clone()]
    } else {
        configured_topics.clone()
    };
    let inspection = inspect_local_stream(path)?;
    let topic_depths = inspection
        .topics
        .into_iter()
        .map(|topic| (topic.topic, topic.message_count))
        .collect::<BTreeMap<_, _>>();
    let mut matches = Vec::new();
    for topic in &topics_scanned {
        let message_count = topic_depths.get(topic).copied().unwrap_or(0);
        for offset in 0..message_count {
            let Some(message) = read_local_message_at(path, topic, offset)? else {
                continue;
            };
            if !local_message_matches_boundary(&message, &args.transaction_id, &args.commit_lsn) {
                continue;
            }
            let next_offset = offset.checked_add(1).ok_or_else(|| {
                CliError::InvalidConfig("local stream offset overflow".to_string())
            })?;
            matches.push(LocalStreamLocateMatch {
                topic: topic.clone(),
                offset,
                next_offset,
                message_kind: local_message_header(&message, "trellara.message_kind")
                    .unwrap_or("strict_transaction")
                    .to_string(),
                source_id: local_message_header(&message, "trellara.source_id").map(str::to_string),
                dataset_id: local_message_header(&message, "trellara.dataset_id")
                    .map(str::to_string),
                commit_lsn: local_message_header(&message, "trellara.commit_lsn")
                    .map(str::to_string),
                partition_id: local_message_header(&message, "trellara.partition_id")
                    .and_then(|value| value.parse::<u32>().ok()),
                partition_count: local_message_header(&message, "trellara.partition_count")
                    .and_then(|value| value.parse::<usize>().ok()),
                key: message.key,
                seek_command: local_seek_command(topic, offset, args),
            });
        }
    }
    let next_commands = matches
        .iter()
        .map(|matched| matched.seek_command.clone())
        .collect::<Vec<_>>();
    let boundary = LocalStreamLocateBoundarySummary::from_matches(config, &matches);
    let exact_boundary = args.commit_lsn.is_some();
    let replay_warnings = replay_warnings(exact_boundary, &boundary, matches.len());
    let replay_safe = replay_warnings.is_empty();

    Ok(LocalStreamLocateSummary {
        root: path.display().to_string(),
        transaction_id: args.transaction_id.clone(),
        commit_lsn: args.commit_lsn.clone(),
        topics_scanned,
        match_count: matches.len(),
        matches,
        boundary,
        exact_boundary,
        replay_safe,
        replay_warnings,
        next_commands,
    })
}

#[cfg(not(feature = "local-stream"))]
pub(crate) fn locate_configured_local_stream_transaction(
    _config: &TrellaraConfig,
    args: &LocalStreamLocateArgs,
) -> Result<LocalStreamLocateSummary> {
    validate_local_stream_locate_boundary(&args.transaction_id, args.commit_lsn.as_deref())?;
    Err(crate::local_stream_feature_disabled())
}

#[cfg(feature = "local-stream")]
fn replay_warnings(
    exact_boundary: bool,
    boundary: &LocalStreamLocateBoundarySummary,
    match_count: usize,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if !exact_boundary {
        warnings.push(
            "commit_lsn is required for replay-safe exact transaction-boundary location"
                .to_string(),
        );
    }
    if !boundary.complete {
        warnings.push(format!("local boundary status is {}", boundary.status));
    }
    if match_count == 0 {
        warnings.push("no matching local stream messages found".to_string());
    }
    warnings
}

#[cfg(feature = "local-stream")]
fn local_seek_command(topic: &str, offset: i64, args: &LocalStreamLocateArgs) -> String {
    let mut command = format!(
        "trellara stream seek-local --config <config> --topic {topic} --next-offset {offset}"
    );
    command.push_str(&format!(" --transaction-id {}", args.transaction_id));
    if let Some(commit_lsn) = &args.commit_lsn {
        command.push_str(&format!(" --commit-lsn {commit_lsn}"));
    }
    command
}

pub(crate) fn validate_local_stream_locate_boundary(
    transaction_id: &str,
    commit_lsn: Option<&str>,
) -> Result<()> {
    if transaction_id.trim().is_empty() {
        return Err(CliError::InvalidConfig(
            "stream.locate.transaction_id must not be empty".to_string(),
        ));
    }
    if transaction_id.trim() != transaction_id {
        return Err(CliError::InvalidConfig(
            "stream.locate.transaction_id must not contain surrounding whitespace".to_string(),
        ));
    }
    let Some(commit_lsn) = commit_lsn else {
        return Ok(());
    };
    if commit_lsn.trim().is_empty() {
        return Err(CliError::InvalidConfig(
            "stream.locate.commit_lsn must not be empty when provided".to_string(),
        ));
    }
    if commit_lsn.trim() != commit_lsn {
        return Err(CliError::InvalidConfig(
            "stream.locate.commit_lsn must not contain surrounding whitespace".to_string(),
        ));
    }
    if !lsn_shape_is_valid(commit_lsn) || parse_lsn(commit_lsn) == 0 {
        return Err(CliError::InvalidConfig(format!(
            "stream.locate.commit_lsn {commit_lsn:?} must be a non-zero PostgreSQL LSN"
        )));
    }
    Ok(())
}
