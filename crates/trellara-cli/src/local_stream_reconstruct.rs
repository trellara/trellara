#[cfg(feature = "local-stream")]
use std::path::PathBuf;

#[cfg(feature = "local-stream")]
use trellara_stream_local::{
    reconstruct_local_barrier_transaction, LocalBarrierReconstructionRequest,
    LocalPartitionChunkOffset,
};

#[cfg(feature = "local-stream")]
use crate::{
    locate_configured_local_stream_transaction, validate_local_stream_locate_boundary, CliError,
    LocalStreamLocateArgs, LocalStreamLocateBoundarySummary, LocalStreamLocateMatch,
    LocalStreamReconstructArgs, LocalStreamReconstructPartitionOffset,
    LocalStreamReconstructSummary, Result, StreamConfig, TrellaraConfig,
};
#[cfg(not(feature = "local-stream"))]
use crate::{
    validate_local_stream_locate_boundary, CliError, LocalStreamReconstructArgs,
    LocalStreamReconstructSummary, Result, TrellaraConfig,
};

#[cfg(feature = "local-stream")]
pub(crate) fn reconstruct_configured_local_stream_transaction(
    config: &TrellaraConfig,
    args: &LocalStreamReconstructArgs,
) -> Result<LocalStreamReconstructSummary> {
    validate_local_stream_reconstruct_boundary(args)?;
    let path = match &config.stream {
        StreamConfig::Local { path, .. } => path,
        StreamConfig::Kafka { .. } => {
            return Err(CliError::InvalidConfig(
                "stream.kind must be local for local stream reconstruction".to_string(),
            ));
        }
    };
    let locate = locate_configured_local_stream_transaction(
        config,
        &LocalStreamLocateArgs {
            config: PathBuf::new(),
            transaction_id: args.transaction_id.clone(),
            commit_lsn: args.commit_lsn.clone(),
            topic: None,
        },
    )?;
    if !locate.boundary.complete {
        return Err(CliError::InvalidConfig(format!(
            "cannot reconstruct incomplete local stream boundary for transaction {}: {}{}",
            args.transaction_id,
            locate.boundary.status,
            boundary_refusal_detail(&locate.boundary)
        )));
    }

    let manifest = required_match(&locate.matches, "manifest")?;
    let commit = required_match(&locate.matches, "commit_marker")?;
    let partition_offsets = partition_offsets(&locate.matches);
    let reconstructed = reconstruct_local_barrier_transaction(
        path,
        &LocalBarrierReconstructionRequest {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            manifest_offset: manifest.offset,
            commit_offset: commit.offset,
            partition_offsets: partition_offsets
                .iter()
                .map(|offset| LocalPartitionChunkOffset {
                    partition_id: offset.partition_id,
                    offset: offset.offset,
                })
                .collect(),
        },
    )?;

    Ok(LocalStreamReconstructSummary {
        root: path.display().to_string(),
        transaction_id: args.transaction_id.clone(),
        commit_lsn: args.commit_lsn.clone(),
        boundary: locate.boundary,
        manifest_offset: manifest.offset,
        commit_offset: commit.offset,
        partition_offsets,
        partition_chunk_count: reconstructed.chunks.len(),
        reconstructed_change_count: reconstructed.changes.len(),
        source_order: reconstructed
            .changes
            .iter()
            .map(|change| change.total_order)
            .collect(),
        proof: "manifest, commit marker, and every partition chunk reconstructed before acknowledgement"
            .to_string(),
    })
}

#[cfg(not(feature = "local-stream"))]
pub(crate) fn reconstruct_configured_local_stream_transaction(
    _config: &TrellaraConfig,
    args: &LocalStreamReconstructArgs,
) -> Result<LocalStreamReconstructSummary> {
    validate_local_stream_reconstruct_boundary(args)?;
    Err(crate::local_stream_feature_disabled())
}

pub(crate) fn validate_local_stream_reconstruct_boundary(
    args: &LocalStreamReconstructArgs,
) -> Result<()> {
    let Some(commit_lsn) = args.commit_lsn.as_deref() else {
        return Err(CliError::InvalidConfig(
            "stream.reconstruct.commit_lsn is required for exact transaction reconstruction"
                .to_string(),
        ));
    };
    validate_local_stream_locate_boundary(&args.transaction_id, Some(commit_lsn))
}

#[cfg(feature = "local-stream")]
fn required_match<'a>(
    matches: &'a [LocalStreamLocateMatch],
    message_kind: &str,
) -> Result<&'a LocalStreamLocateMatch> {
    matches
        .iter()
        .find(|matched| matched.message_kind == message_kind)
        .ok_or_else(|| {
            CliError::InvalidConfig(format!("local stream boundary is missing {message_kind}"))
        })
}

#[cfg(feature = "local-stream")]
fn partition_offsets(
    matches: &[LocalStreamLocateMatch],
) -> Vec<LocalStreamReconstructPartitionOffset> {
    matches
        .iter()
        .filter(|matched| matched.message_kind == "partition_chunk")
        .filter_map(|matched| {
            matched
                .partition_id
                .map(|partition_id| LocalStreamReconstructPartitionOffset {
                    partition_id,
                    offset: matched.offset,
                })
        })
        .collect()
}

#[cfg(feature = "local-stream")]
fn boundary_refusal_detail(boundary: &LocalStreamLocateBoundarySummary) -> String {
    let mut details = Vec::new();
    if !boundary.missing_message_kinds.is_empty() {
        details.push(format!(
            "missing message kinds: {}",
            boundary.missing_message_kinds.join(", ")
        ));
    }
    if !boundary.metadata_conflicts.is_empty() {
        details.push(format!(
            "metadata conflicts: {}",
            boundary.metadata_conflicts.join("; ")
        ));
    }
    if !boundary.missing_partition_ids.is_empty() {
        details.push(format!(
            "missing partition ids: {}",
            boundary
                .missing_partition_ids
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    match &details[..] {
        [] => String::new(),
        _ => format!(" ({})", details.join("; ")),
    }
}
