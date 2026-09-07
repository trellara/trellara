use std::collections::BTreeMap;

use trellara_protocol::{parse_lsn, TransactionEnvelope};

use crate::{Result, TrellaraConfig};

pub(crate) const DETERMINISTIC_EPOCH_ID: &str = "<deterministic>";

pub(crate) fn resolve_lake_epoch_id(
    requested_epoch_id: &str,
    config: &TrellaraConfig,
    envelopes: &[TransactionEnvelope],
) -> Result<String> {
    if requested_epoch_id != DETERMINISTIC_EPOCH_ID {
        return Ok(requested_epoch_id.to_string());
    }

    let mut windows = BTreeMap::<String, SourceWindow>::new();
    for envelope in envelopes {
        if envelope.dataset_id != config.dataset.id {
            continue;
        }
        if envelope.source_id != config.source.id {
            continue;
        }
        let window = windows.entry(envelope.source_id.clone()).or_default();
        window.advance(&envelope.commit_lsn)?;
    }

    let source_windows = windows
        .into_iter()
        .map(|(source_id, window)| {
            trellara_lake::LakeEpochSourceWindow::new(source_id, window.start_lsn, window.end_lsn)
        })
        .collect::<Vec<_>>();

    Ok(trellara_lake::deterministic_epoch_id(
        &config.dataset.id,
        [config.source.id.as_str()],
        &trellara_lake::LakeStragglerPolicy::WaitAllRequired,
        &source_windows,
    )?)
}

#[derive(Clone, Debug, Default)]
struct SourceWindow {
    start_lsn: Option<String>,
    end_lsn: Option<String>,
}

impl SourceWindow {
    fn advance(&mut self, commit_lsn: &str) -> Result<()> {
        let commit_lsn_value = parse_lsn(commit_lsn)?;
        if self
            .start_lsn
            .as_deref()
            .map(parse_lsn)
            .transpose()?
            .is_none_or(|current| commit_lsn_value < current)
        {
            self.start_lsn = Some(commit_lsn.to_string());
        }
        if self
            .end_lsn
            .as_deref()
            .map(parse_lsn)
            .transpose()?
            .is_none_or(|current| commit_lsn_value > current)
        {
            self.end_lsn = Some(commit_lsn.to_string());
        }
        Ok(())
    }
}
