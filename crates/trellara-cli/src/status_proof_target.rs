use crate::{
    validation_stale_message, ChecksumStatus, CorrectnessProofCheck, CorrectnessProofInputs,
    TargetSourceProgress, ValidationProgress,
};

pub(crate) fn target_checkpoint_proof_check(
    input: &CorrectnessProofInputs<'_>,
) -> CorrectnessProofCheck {
    match (input.status.source.as_ref(), input.status.target.as_ref()) {
        (Some(source), Some(target)) if input.target_caught_up => CorrectnessProofCheck::verified(
            "target_checkpoint",
            format!(
                "target applied checkpoint {} has caught up to target durable LSN {} and source durable LSN {}",
                target.last_applied_lsn, target.last_durable_lsn, source.last_durable_lsn
            ),
        ),
        (Some(source), Some(target)) if target.target_is_caught_up => {
            let progress = TargetSourceProgress::from_status(input.status);
            CorrectnessProofCheck::at_risk(
                "target_checkpoint",
                format!(
                    "target applied checkpoint {} is caught up to target durable LSN {} but behind source durable LSN {} by {} bytes",
                    target.last_applied_lsn,
                    target.last_durable_lsn,
                    source.last_durable_lsn,
                    progress.source_to_target_lag_bytes.unwrap_or_default()
                ),
                "run trellara relay and trellara apply until target applied watermark reaches the source durable watermark",
            )
        }
        (Some(_source), Some(target)) => CorrectnessProofCheck::at_risk(
            "target_checkpoint",
            format!(
                "target applied checkpoint {} is behind durable LSN {} by {} bytes",
                target.last_applied_lsn, target.last_durable_lsn, target.durable_to_applied_bytes
            ),
            "run trellara apply until target applied watermark catches up",
        ),
        (None, Some(target)) => CorrectnessProofCheck::missing_evidence(
            "target_checkpoint",
            format!(
                "target checkpoint {} cannot be proven against missing source durable watermark",
                target.last_applied_lsn
            ),
            "run trellara relay and trellara status until source checkpoint evidence exists",
        ),
        (_, None) => CorrectnessProofCheck::missing_evidence(
            "target_checkpoint",
            "target checkpoint evidence is missing",
            "run trellara apply-schema and trellara apply to establish a target checkpoint",
        ),
    }
}

pub(crate) fn target_quarantine_proof_check(
    input: &CorrectnessProofInputs<'_>,
) -> CorrectnessProofCheck {
    if input.no_target_quarantine {
        CorrectnessProofCheck::verified(
            "target_quarantine",
            "target quarantine has no blocked transaction",
        )
    } else {
        let quarantine = input
            .status
            .latest_quarantine
            .as_ref()
            .expect("checked quarantine");
        CorrectnessProofCheck::at_risk(
            "target_quarantine",
            format!(
                "target quarantine contains transaction {} at LSN {}: {}",
                quarantine.transaction_id, quarantine.commit_lsn, quarantine.reason
            ),
            "run trellara quarantine list, repair the target contract, then trellara quarantine replay-ready before redelivery",
        )
    }
}

pub(crate) fn checksum_validation_proof_check(
    latest_checksum_status: ChecksumStatus,
    validation_progress: ValidationProgress,
) -> CorrectnessProofCheck {
    match latest_checksum_status {
        ChecksumStatus::Match if validation_progress.is_current => CorrectnessProofCheck::verified(
            "checksum_validation",
            "latest validation checksums match",
        ),
        ChecksumStatus::Match => CorrectnessProofCheck::at_risk(
            "checksum_validation",
            validation_stale_message(validation_progress),
            "run trellara verify until validation watermarks reach the current source and target checkpoints",
        ),
        ChecksumStatus::Mismatch => CorrectnessProofCheck::at_risk(
            "checksum_validation",
            "latest validation found checksum drift",
            "run trellara verify after repair; use trellara reseed if checksum drift remains",
        ),
        ChecksumStatus::Unknown => CorrectnessProofCheck::missing_evidence(
            "checksum_validation",
            "validation checksum evidence is missing",
            "run trellara verify to produce checksum evidence",
        ),
    }
}
