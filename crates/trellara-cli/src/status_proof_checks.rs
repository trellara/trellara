use crate::{
    checksum_validation_proof_check, is_partitioned_mode, is_strict_chunked_mode,
    snapshot_handoff_proof_check, source_proof_checks, target_checkpoint_proof_check,
    target_quarantine_proof_check, transaction_boundary_proof_evidence, CorrectnessProofCheck,
    CorrectnessProofInputs, TransactionBoundaryStatus,
};

pub(crate) fn correctness_proof_checks(
    input: CorrectnessProofInputs<'_>,
) -> Vec<CorrectnessProofCheck> {
    let mut checks = Vec::new();
    let status = input.status;

    checks.extend(source_proof_checks(&input));

    checks.push(target_checkpoint_proof_check(&input));

    checks.push(match input.transaction_boundary.status {
        TransactionBoundaryStatus::Verified => CorrectnessProofCheck::verified(
            "transaction_boundary",
            transaction_boundary_proof_evidence(input.transaction_boundary),
        ),
        TransactionBoundaryStatus::AtRisk => CorrectnessProofCheck::at_risk(
            "transaction_boundary",
            transaction_boundary_proof_evidence(input.transaction_boundary),
            "repair lagging or quarantined transaction evidence before relying on the boundary",
        ),
        TransactionBoundaryStatus::PendingEvidence => CorrectnessProofCheck::missing_evidence(
            "transaction_boundary",
            transaction_boundary_proof_evidence(input.transaction_boundary),
            "run the relevant applier and status commands until checkpoint evidence exists",
        ),
    });

    if is_partitioned_mode(&status.mode) {
        checks.push(if status.partition_watermarks.is_none() {
            CorrectnessProofCheck::missing_evidence(
                "partition_manifest_barrier",
                "partition watermark evidence is missing",
                "run trellara partition-watermarks after the barrier-aware applier has processed every partition",
            )
        } else if input.partition_watermark_ready {
            CorrectnessProofCheck::verified(
                "partition_manifest_barrier",
                "all partition checkpoints are present, global applied watermark is caught up, and manifest checksum/event-count evidence headers are validated",
            )
        } else {
            CorrectnessProofCheck::at_risk(
                "partition_manifest_barrier",
                "partition watermarks are incomplete, globally lagging, or missing validated manifest checksum/event-count evidence",
                "run the barrier-aware applier for all partition topics until partition watermarks are complete and caught up",
            )
        });
    }

    if is_strict_chunked_mode(&status.mode) {
        checks.push(
            if status.target.is_none() {
                CorrectnessProofCheck::missing_evidence(
                    "strict_chunk_manifest_barrier",
                    "target checkpoint evidence is missing for strict chunk manifest and commit marker apply",
                    "run the barrier-aware applier until strict chunks are reconstructed and applied",
                )
            } else if input.target_caught_up && input.no_target_quarantine {
                CorrectnessProofCheck::verified(
                    "strict_chunk_manifest_barrier",
                    "target checkpoint caught up after strict chunk manifest and commit marker reconstruction",
                )
            } else {
                CorrectnessProofCheck::at_risk(
                    "strict_chunk_manifest_barrier",
                    "strict chunk manifest and commit marker apply is lagging or blocked by target quarantine",
                    "run trellara apply and clear any target quarantine before relying on the strict chunk barrier",
                )
            },
        );
    }

    checks.push(snapshot_handoff_proof_check(
        status,
        input.snapshot_handoff_status,
    ));

    checks.push(target_quarantine_proof_check(&input));

    checks.push(checksum_validation_proof_check(
        input.latest_checksum_status,
        input.validation_progress,
    ));

    checks
}
