use crate::lsn_validation::validate_required_nonzero_lsn;
use crate::validation_event_digest::validate_evidence_digest;
use crate::validation_guards::{require_non_empty, require_non_negative, require_positive};
use crate::{CheckpointError, Result, ValidationEvent};

pub(crate) fn validate_validation_event(event: &ValidationEvent) -> Result<()> {
    require_non_empty(
        &event.source_id,
        "validation event source_id must not be empty",
    )?;
    require_non_empty(
        &event.dataset_id,
        "validation event dataset_id must not be empty",
    )?;
    require_non_empty(
        &event.source_watermark_lsn,
        "validation event source watermark LSN must not be empty",
    )?;
    require_non_empty(
        &event.target_watermark_lsn,
        "validation event target watermark LSN must not be empty",
    )?;
    validate_required_nonzero_lsn(
        "validation event",
        "source_watermark_lsn",
        &event.source_watermark_lsn,
    )?;
    validate_required_nonzero_lsn(
        "validation event",
        "target_watermark_lsn",
        &event.target_watermark_lsn,
    )?;
    require_positive(
        event.table_count,
        format!(
            "validation event must compare at least one table, got {}",
            event.table_count
        ),
    )?;
    require_non_negative(
        event.drift_count,
        format!(
            "validation event cannot record negative drift count {}",
            event.drift_count
        ),
    )?;
    validate_drift_counts(event)?;
    validate_convergence(event)?;
    validate_evidence_digest(event)?;

    Ok(())
}

fn validate_drift_counts(event: &ValidationEvent) -> Result<()> {
    if event.drift_count > event.table_count {
        return Err(CheckpointError::Store(format!(
            "validation event drift count {} cannot exceed table count {}",
            event.drift_count, event.table_count
        )));
    }
    let drift_relation_count = drift_relation_count_len(event.drift_relations.len())?;
    if drift_relation_count != event.drift_count {
        return Err(CheckpointError::Store(format!(
            "validation event drift relation count {} does not match drift count {}",
            event.drift_relations.len(),
            event.drift_count
        )));
    }
    if event
        .drift_relations
        .iter()
        .any(|relation| relation.trim().is_empty())
    {
        return Err(CheckpointError::Store(
            "validation event drift relations must not be empty".to_string(),
        ));
    }

    Ok(())
}

fn drift_relation_count_len(len: usize) -> Result<i64> {
    i64::try_from(len).map_err(|_| {
        CheckpointError::Store(format!(
            "validation event drift relation count {len} exceeds supported count range"
        ))
    })
}

fn validate_convergence(event: &ValidationEvent) -> Result<()> {
    if !event.converged {
        validate_non_convergence_has_evidence(event)?;
        return Ok(());
    }
    if event.drift_count != 0 {
        return Err(CheckpointError::Store(
            "converged validation event cannot include drift".to_string(),
        ));
    }
    if event.source_watermark_lsn != event.target_watermark_lsn {
        return Err(CheckpointError::Store(format!(
            "converged validation event requires matching watermarks, got source {} and target {}",
            event.source_watermark_lsn, event.target_watermark_lsn
        )));
    }

    Ok(())
}

fn validate_non_convergence_has_evidence(event: &ValidationEvent) -> Result<()> {
    if event.drift_count == 0 && event.source_watermark_lsn == event.target_watermark_lsn {
        return Err(CheckpointError::Store(
            "non-converged validation event must include drift or mismatched watermarks"
                .to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drift_relation_count_len_accepts_normal_relation_counts() {
        assert_eq!(drift_relation_count_len(42).expect("count"), 42);
    }
}
