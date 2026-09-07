use std::collections::BTreeSet;

use crate::LakeEpochSummary;

pub(crate) fn validate_source_watermark_rows(epoch: &LakeEpochSummary) -> Result<(), String> {
    let mut seen_sources = BTreeSet::new();
    for source in &epoch.source_watermarks {
        if source.source_id.trim().is_empty() || source.source_id.trim() != source.source_id {
            return Err(format!(
                "source watermark has invalid source_id {:?}",
                source.source_id
            ));
        }
        if !seen_sources.insert(source.source_id.clone()) {
            return Err(format!(
                "source watermark has duplicate source_id {}",
                source.source_id
            ));
        }
        if !source_state_is_known(&source.state) {
            return Err(format!(
                "source {} has unknown watermark state {}",
                source.source_id, source.state
            ));
        }
        if source_requires_end_lsn(&source.state) && optional_text_is_empty(&source.end_lsn) {
            return Err(format!(
                "source {} state {} requires end_lsn evidence",
                source.source_id, source.state
            ));
        }
        if source_is_gap_state(&source.state) && optional_text_is_empty(&source.gap_reason) {
            return Err(format!(
                "source {} state {} requires gap_reason evidence",
                source.source_id, source.state
            ));
        }
    }
    Ok(())
}

fn source_state_is_known(state: &str) -> bool {
    matches!(
        state,
        "complete" | "lagging" | "missing" | "quarantined" | "reseeding"
    )
}

fn source_requires_end_lsn(state: &str) -> bool {
    matches!(state, "complete" | "lagging")
}

fn source_is_gap_state(state: &str) -> bool {
    matches!(state, "lagging" | "missing" | "quarantined" | "reseeding")
}

fn optional_text_is_empty(value: &Option<String>) -> bool {
    value.as_deref().map(str::is_empty).unwrap_or(true)
}
