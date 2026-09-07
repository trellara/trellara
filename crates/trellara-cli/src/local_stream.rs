#[cfg(feature = "local-stream")]
#[path = "local_stream_seek_boundary.rs"]
mod local_stream_seek_boundary;

#[cfg(feature = "local-stream")]
use local_stream_seek_boundary::{
    local_seek_boundary_context, seek_boundary_warnings, seek_replay_warnings,
    verify_seek_expected_boundary,
};

#[cfg(feature = "local-stream")]
use trellara_stream_local::{inspect_local_stream, set_local_cursor_with_policy};

#[cfg(feature = "local-stream")]
use crate::StreamConfig;
use crate::{CliError, LocalStreamSeekArgs, LocalStreamSeekSummary, Result, TrellaraConfig};

#[cfg(feature = "local-stream")]
pub(crate) fn seek_configured_local_stream(
    config: &TrellaraConfig,
    args: &LocalStreamSeekArgs,
) -> Result<LocalStreamSeekSummary> {
    validate_local_stream_seek_args(args)?;
    let path = match &config.stream {
        StreamConfig::Local { path, .. } => path,
        StreamConfig::Kafka { .. } => {
            return Err(CliError::InvalidConfig(
                "stream.kind must be local for local stream cursor seek".to_string(),
            ));
        }
    };
    let configured_topics = config.local_stream_topics()?;
    if !configured_topics.iter().any(|topic| topic == &args.topic) {
        return Err(CliError::InvalidConfig(format!(
            "topic {} is not configured for this flow; configured topics: {}",
            args.topic,
            configured_topics.join(", ")
        )));
    }
    let group_id = args.consumer_group.clone().unwrap_or_else(|| {
        format!(
            "trellara-applier-{}-{}",
            config.source.id, config.dataset.id
        )
    });
    let inspection_before = inspect_local_stream(path)?;
    let topic_message_count = inspection_before
        .topics
        .iter()
        .find(|topic| topic.topic == args.topic)
        .map(|topic| topic.message_count)
        .unwrap_or(0);
    let previous_next_offset = inspection_before
        .cursors
        .iter()
        .find(|cursor| cursor.group_id == group_id && cursor.topic == args.topic)
        .map(|cursor| cursor.next_offset);
    let pending_before =
        previous_next_offset.map(|offset| topic_message_count.saturating_sub(offset).max(0));
    let boundary_context =
        local_seek_boundary_context(config, path, &args.topic, args.next_offset)?;
    verify_seek_expected_boundary(
        path,
        &args.topic,
        args.next_offset,
        args.transaction_id.as_deref(),
        args.commit_lsn.as_deref(),
    )?;
    let cursor = set_local_cursor_with_policy(
        path,
        group_id.clone(),
        args.topic.clone(),
        args.next_offset,
        args.allow_ahead,
    )?;
    let pending_after = topic_message_count
        .saturating_sub(cursor.next_offset)
        .max(0);
    let redelivered_messages = previous_next_offset
        .map(|offset| offset.saturating_sub(cursor.next_offset).max(0))
        .unwrap_or(0);
    let skipped_messages = previous_next_offset
        .map(|offset| cursor.next_offset.saturating_sub(offset).max(0))
        .unwrap_or(0);
    let movement = match previous_next_offset {
        None => "initialized",
        Some(previous) if cursor.next_offset < previous => "rewind_replay",
        Some(previous) if cursor.next_offset > previous => "fast_forward_skip",
        Some(_) => "unchanged",
    }
    .to_string();
    let cursor_status = if cursor.next_offset > topic_message_count {
        "ahead_of_topic"
    } else {
        "ok"
    }
    .to_string();
    let boundary_warnings = boundary_context
        .as_ref()
        .map(|context| seek_boundary_warnings(&context.boundary))
        .unwrap_or_default();
    let replay_warnings = seek_replay_warnings(
        &movement,
        args.transaction_id.as_deref(),
        args.commit_lsn.as_deref(),
        boundary_context.as_ref().map(|context| &context.boundary),
    );
    let replay_safe = replay_warnings.is_empty();

    Ok(LocalStreamSeekSummary {
        root: path.display().to_string(),
        group_id: cursor.group_id,
        topic: cursor.topic,
        previous_next_offset,
        next_offset: cursor.next_offset,
        topic_message_count,
        pending_before,
        pending_after,
        movement,
        redelivered_messages,
        skipped_messages,
        cursor_status,
        allow_ahead: args.allow_ahead,
        configured_topics,
        boundary: boundary_context
            .as_ref()
            .map(|context| context.boundary.clone()),
        boundary_warnings,
        replay_safe,
        replay_warnings,
        boundary_seek_commands: boundary_context
            .map(|context| context.seek_commands)
            .unwrap_or_default(),
    })
}

#[cfg(not(feature = "local-stream"))]
pub(crate) fn seek_configured_local_stream(
    _config: &TrellaraConfig,
    args: &LocalStreamSeekArgs,
) -> Result<LocalStreamSeekSummary> {
    validate_local_stream_seek_args(args)?;
    Err(crate::local_stream_feature_disabled())
}

pub(crate) fn validate_local_stream_seek_args(args: &LocalStreamSeekArgs) -> Result<()> {
    if args.next_offset < 0 {
        return Err(CliError::InvalidConfig(format!(
            "stream.seek.next_offset must be non-negative, got {}",
            args.next_offset
        )));
    }
    validate_optional_seek_boundary(args.transaction_id.as_deref(), args.commit_lsn.as_deref())?;
    Ok(())
}

fn validate_optional_seek_boundary(
    transaction_id: Option<&str>,
    commit_lsn: Option<&str>,
) -> Result<()> {
    if let Some(transaction_id) = transaction_id {
        if transaction_id.trim().is_empty() {
            return Err(CliError::InvalidConfig(
                "stream.seek.transaction_id must not be empty when provided".to_string(),
            ));
        }
        if transaction_id.trim() != transaction_id {
            return Err(CliError::InvalidConfig(
                "stream.seek.transaction_id must not contain surrounding whitespace".to_string(),
            ));
        }
        if transaction_id.split_whitespace().count() > 1 {
            return Err(CliError::InvalidConfig(
                "stream.seek.transaction_id must not contain whitespace".to_string(),
            ));
        }
    }
    if let Some(commit_lsn) = commit_lsn {
        crate::validate_local_stream_locate_boundary(
            transaction_id.unwrap_or("trellara-seek-boundary"),
            Some(commit_lsn),
        )?;
    }
    Ok(())
}
