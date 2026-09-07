use std::path::PathBuf;

use trellara_checkpoint::ApplyQuarantine;

use crate::{
    locate_configured_local_stream_transaction, DatasetMode, LocalStreamLocateArgs,
    LocalStreamLocateBoundarySummary, Result, StreamConfig, TrellaraConfig,
};

pub(crate) fn quarantine_recovery_hint(
    config: &TrellaraConfig,
    quarantine: &ApplyQuarantine,
    topics: &[String],
) -> String {
    let redelivery_hint = replay_redelivery_hint(config, topics);
    if quarantine.reason == "no_rows_matched" {
        return format!(
            "the target row was missing or diverged for transaction {} at {}; run trellara verify and reseed the affected table if needed before marking replay-ready. After repair, {}",
            quarantine.transaction_id, quarantine.commit_lsn, redelivery_hint
        );
    }
    redelivery_hint
}

pub(crate) fn replay_redelivery_hint(config: &TrellaraConfig, topics: &[String]) -> String {
    if config.dataset.strict_chunking.is_some() {
        return format!(
            "seek or redeliver the strict chunk messages plus manifest and commit marker barrier on {}; the barrier-aware applier will reconstruct the complete transaction before apply and will not skip it as a duplicate",
            topics.join(", ")
        );
    }

    match config.dataset.mode {
        DatasetMode::StrictTransactionOrder => format!(
            "seek or redeliver the strict transaction message on {}; the applier will not skip it as a duplicate",
            topics.join(", ")
        ),
        DatasetMode::PartitionedScaleMode => format!(
            "seek or redeliver the manifest, commit marker, and participating partition messages on {}; the barrier-aware applier will not skip the transaction as a duplicate",
            topics.join(", ")
        ),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReplayRedeliveryPlan {
    pub(crate) commands: Vec<String>,
    pub(crate) exact_seek_commands: Vec<String>,
    pub(crate) boundary: Option<LocalStreamLocateBoundarySummary>,
    pub(crate) warnings: Vec<String>,
}

pub(crate) fn replay_redelivery_plan(
    config: &TrellaraConfig,
    topics: &[String],
    boundary: Option<(&str, &str)>,
) -> Result<ReplayRedeliveryPlan> {
    match &config.stream {
        StreamConfig::Local { .. } => {
            let mut commands = Vec::new();
            let mut exact_seek_commands = Vec::new();
            let mut redelivery_boundary = None;
            let mut warnings = Vec::new();
            if let Some((transaction_id, commit_lsn)) = boundary {
                commands.push(format!(
                    "trellara stream locate-local --config <config> --transaction-id {transaction_id} --commit-lsn {commit_lsn}"
                ));
                let locate = locate_configured_local_stream_transaction(
                    config,
                    &LocalStreamLocateArgs {
                        config: PathBuf::from("<config>"),
                        transaction_id: transaction_id.to_string(),
                        commit_lsn: Some(commit_lsn.to_string()),
                        topic: None,
                    },
                )?;
                redelivery_boundary = Some(locate.boundary.clone());
                if locate.boundary.complete {
                    exact_seek_commands = locate.next_commands;
                } else {
                    warnings = redelivery_boundary_warnings(&locate.boundary);
                }
            }
            if exact_seek_commands.is_empty() {
                commands.extend(topics.iter().map(|topic| {
                    format!(
                        "trellara stream seek-local --config <config> --topic {topic} --next-offset <offset>"
                    )
                }));
            } else {
                commands.extend(exact_seek_commands.iter().cloned());
            }
            Ok(ReplayRedeliveryPlan {
                commands,
                exact_seek_commands,
                boundary: redelivery_boundary,
                warnings,
            })
        }
        StreamConfig::Kafka { .. } => Ok(ReplayRedeliveryPlan {
            commands: Vec::new(),
            exact_seek_commands: Vec::new(),
            boundary: None,
            warnings: Vec::new(),
        }),
    }
}

fn redelivery_boundary_warnings(boundary: &LocalStreamLocateBoundarySummary) -> Vec<String> {
    let mut warnings = vec![format!(
        "local redelivery boundary is {}; exact seek commands were not generated",
        boundary.status
    )];
    if !boundary.missing_message_kinds.is_empty() {
        warnings.push(format!(
            "missing message kinds: {}",
            boundary.missing_message_kinds.join(", ")
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
