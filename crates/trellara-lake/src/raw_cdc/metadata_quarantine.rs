use crate::raw_cdc_types::{LakeRawCdcEpochQuarantineRow, LakeRawCdcEpochSourceRow};
use crate::LakeEpochSourceState;

pub(crate) fn quarantine_rows_for_sources(
    epoch_id: &str,
    sources: &[LakeRawCdcEpochSourceRow],
) -> Vec<LakeRawCdcEpochQuarantineRow> {
    sources
        .iter()
        .filter(|source| source.state == LakeEpochSourceState::Quarantined)
        .map(|source| source_quarantine_row(epoch_id, source))
        .collect()
}

fn source_quarantine_row(
    epoch_id: &str,
    source: &LakeRawCdcEpochSourceRow,
) -> LakeRawCdcEpochQuarantineRow {
    LakeRawCdcEpochQuarantineRow {
        epoch_id: epoch_id.to_string(),
        source_id: Some(source.source_id.clone()),
        transaction_id: None,
        commit_lsn: source_commit_lsn(source),
        reason: source_quarantine_reason(source),
        details: source.lag_reason.clone(),
        recovery_command: Some(
            "trellara lake fanin verify --stream-epoch <stream-epoch.json> --lake-epoch <lake-epoch.json>"
                .to_string(),
        ),
    }
}

fn source_commit_lsn(source: &LakeRawCdcEpochSourceRow) -> Option<String> {
    (!source.end_lsn.is_empty()).then(|| source.end_lsn.clone())
}

fn source_quarantine_reason(source: &LakeRawCdcEpochSourceRow) -> String {
    if source.transaction_count == 0 {
        "source_gap_quarantined".to_string()
    } else {
        "source_evidence_quarantined".to_string()
    }
}
