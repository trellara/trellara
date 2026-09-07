use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::index::{checked_offset, last_valid_offset, recover_topic_index};
use crate::index_read::read_record_at;
use crate::paths::{topic_dir, topic_index_path};
use crate::{
    validate_topic, LocalDurability, LocalStreamError, LocalTopicInspection, LocalTopicProofHeader,
    Result,
};

pub(crate) fn inspect_topics(root: &Path) -> Result<Vec<LocalTopicInspection>> {
    let dir = topic_dir(root);
    let mut topics = Vec::new();
    match fs::read_dir(&dir) {
        Ok(entries) => {
            for entry in entries {
                let entry = entry.map_err(|source| LocalStreamError::Io {
                    path: dir.display().to_string(),
                    source,
                })?;
                inspect_topic_entry(root, &entry, &mut topics)?;
            }
        }
        Err(source) if source.kind() == ErrorKind::NotFound => {}
        Err(source) => {
            return Err(LocalStreamError::Io {
                path: dir.display().to_string(),
                source,
            });
        }
    }
    topics.sort_by(|left, right| left.topic.cmp(&right.topic));
    Ok(topics)
}

fn inspect_topic_entry(
    root: &Path,
    entry: &fs::DirEntry,
    topics: &mut Vec<LocalTopicInspection>,
) -> Result<()> {
    let path = entry.path();
    if !path.is_file() || path.extension().and_then(|value| value.to_str()) != Some("log") {
        return Ok(());
    }
    let Some(topic) = path.file_stem().and_then(|value| value.to_str()) else {
        return Ok(());
    };
    validate_topic(topic)?;
    let state = recover_topic_index(
        &path,
        &topic_index_path(root, topic),
        LocalDurability::Buffered,
    )?;
    let file_bytes = entry
        .metadata()
        .map_err(|source| LocalStreamError::Io {
            path: path.display().to_string(),
            source,
        })?
        .len();
    let last_valid_offset = last_valid_offset(state.positions.len())?;
    topics.push(LocalTopicInspection {
        topic: topic.to_string(),
        message_count: checked_offset(state.positions.len())?,
        last_valid_offset,
        last_proof_headers: last_proof_headers(
            &path,
            &topic_index_path(root, topic),
            topic,
            last_valid_offset,
        )?,
        replayable: !state.positions.is_empty(),
        valid_bytes: state.valid_end,
        file_bytes,
        torn_tail_bytes: file_bytes.saturating_sub(state.valid_end),
        index_entries: checked_offset(state.positions.len())?,
        index_bytes: fs::metadata(topic_index_path(root, topic))
            .map(|metadata| metadata.len())
            .unwrap_or(0),
        index_status: state.index_status,
    });
    Ok(())
}

fn last_proof_headers(
    path: &Path,
    index_path: &Path,
    topic: &str,
    last_valid_offset: Option<i64>,
) -> Result<Vec<LocalTopicProofHeader>> {
    let Some(offset) = last_valid_offset else {
        return Ok(Vec::new());
    };
    let Some(message) = read_record_at(path, index_path, topic, offset, LocalDurability::Buffered)?
    else {
        return Ok(Vec::new());
    };
    Ok(message
        .headers
        .into_iter()
        .filter(|header| proof_header_key(&header.key))
        .map(|header| LocalTopicProofHeader {
            key: header.key,
            value: header.value,
        })
        .collect())
}

fn proof_header_key(key: &str) -> bool {
    matches!(
        key,
        "trellara.commit_lsn"
            | "trellara.message_kind"
            | "trellara.partitioned_scale_decision"
            | "trellara.requires_ddl_barrier"
            | "trellara.dml_replay_after_ddl_barrier_required"
            | "trellara.ddl_release_gates"
            | "trellara.ddl_propagation_decisions"
            | "trellara.ddl_target_ack_required"
            | "trellara.ddl_propagation_policy_sha256"
    )
}
