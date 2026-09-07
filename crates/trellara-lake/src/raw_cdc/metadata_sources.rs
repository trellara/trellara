use std::collections::BTreeSet;

use super::RawCdcMetadataInput;
use crate::raw_cdc_types::LakeRawCdcEpochSourceRow;
use crate::{LakeEpochSourceState, LakeError, LakeStragglerPolicy};

pub(crate) struct SourceCompletenessCounts {
    pub(crate) required_source_count: usize,
    pub(crate) complete_source_count: usize,
    pub(crate) missing_source_count: usize,
    pub(crate) quarantined_source_count: usize,
}

pub(crate) fn add_required_gap_source_rows(input: &mut RawCdcMetadataInput) {
    let observed_sources = input
        .source_rows
        .iter()
        .map(|source| source.source_id.clone())
        .collect::<BTreeSet<_>>();
    for source_id in input.required_sources.difference(&observed_sources) {
        let (state, lag_reason) = missing_source_state(&input.straggler_policy);
        input.source_rows.push(LakeRawCdcEpochSourceRow {
            epoch_id: input.epoch_id.clone(),
            source_id: source_id.clone(),
            state,
            start_lsn: String::new(),
            end_lsn: String::new(),
            transaction_count: 0,
            change_count: 0,
            checksum_rollup: 0,
            lag_reason: Some(lag_reason.to_string()),
        });
    }
    input
        .source_rows
        .sort_by(|left, right| left.source_id.cmp(&right.source_id));
}

pub(crate) fn validate_source_gap_evidence(input: &RawCdcMetadataInput) -> Result<(), LakeError> {
    for source in &input.source_rows {
        validate_optional_lag_reason(source)?;
        if source_requires_gap_reason(source.state) && source.lag_reason.is_none() {
            return Err(LakeError::InvalidRawCdcEpochMetadataField {
                field: "source_rows[].lag_reason",
                reason: format!(
                    "source {} state {:?} requires explicit lag reason evidence",
                    source.source_id, source.state
                ),
            });
        }
    }
    Ok(())
}

fn validate_optional_lag_reason(source: &LakeRawCdcEpochSourceRow) -> Result<(), LakeError> {
    let Some(reason) = &source.lag_reason else {
        return Ok(());
    };
    if reason.trim().is_empty() {
        return Err(LakeError::InvalidRawCdcEpochMetadataField {
            field: "source_rows[].lag_reason",
            reason: "must not be empty".to_string(),
        });
    }
    if reason != reason.trim() {
        return Err(LakeError::InvalidRawCdcEpochMetadataField {
            field: "source_rows[].lag_reason",
            reason: "must not contain surrounding whitespace".to_string(),
        });
    }
    Ok(())
}

fn source_requires_gap_reason(state: LakeEpochSourceState) -> bool {
    matches!(
        state,
        LakeEpochSourceState::Lagging
            | LakeEpochSourceState::Missing
            | LakeEpochSourceState::Quarantined
            | LakeEpochSourceState::Reseeding
    )
}

pub(crate) fn source_completeness_counts(input: &RawCdcMetadataInput) -> SourceCompletenessCounts {
    let relevant_sources = input
        .source_rows
        .iter()
        .filter(|source| source_is_required_or_unscoped(input, &source.source_id))
        .collect::<Vec<_>>();
    SourceCompletenessCounts {
        required_source_count: required_source_count(input),
        complete_source_count: relevant_sources
            .iter()
            .filter(|source| source.state == LakeEpochSourceState::Complete)
            .count(),
        missing_source_count: relevant_sources
            .iter()
            .filter(|source| {
                matches!(
                    source.state,
                    LakeEpochSourceState::Missing | LakeEpochSourceState::Lagging
                )
            })
            .count(),
        quarantined_source_count: relevant_sources
            .iter()
            .filter(|source| source.state == LakeEpochSourceState::Quarantined)
            .count(),
    }
}

fn required_source_count(input: &RawCdcMetadataInput) -> usize {
    if input.required_sources.is_empty() {
        input.source_rows.len()
    } else {
        input.required_sources.len()
    }
}

fn source_is_required_or_unscoped(input: &RawCdcMetadataInput, source_id: &str) -> bool {
    input.required_sources.is_empty() || input.required_sources.contains(source_id)
}

fn missing_source_state(policy: &LakeStragglerPolicy) -> (LakeEpochSourceState, &'static str) {
    match policy {
        LakeStragglerPolicy::WaitAllRequired => (
            LakeEpochSourceState::Lagging,
            "required source has not reached epoch boundary",
        ),
        LakeStragglerPolicy::PublishWithGaps { .. } => (
            LakeEpochSourceState::Missing,
            "required source missing from published gap epoch",
        ),
        LakeStragglerPolicy::QuarantineOnGap => (
            LakeEpochSourceState::Quarantined,
            "required source missing under quarantine-on-gap policy",
        ),
    }
}
