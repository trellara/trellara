use std::path::Path;

use crate::{
    ApplySummary, ChecksumStatus, RunProofGate, RunProofStatus, TableVerifySummary, VerifySummary,
};
use trellara_verify::ensure_target_caught_up;

pub(crate) fn target_apply_checkpoint_run_proof(
    config_path: &Path,
    apply: &ApplySummary,
) -> RunProofGate {
    let config_display = config_path.display().to_string();
    RunProofGate {
        code: "target_apply_checkpoint".to_string(),
        status: if apply.applied_transactions > 0 && apply.last_commit_lsn.is_some() {
            RunProofStatus::Verified
        } else {
            RunProofStatus::NeedsEvidence
        },
        evidence: format!(
            "{} transactions applied, {} duplicates skipped, last_commit_lsn={}, pending_barrier_transactions={}, missing_manifest={}, missing_commit_marker={}, invalid_commit_marker={}, missing_chunks={}, extra_chunks={}, barrier_pending_blockers={}, barrier_pending_blocker_codes={}, barrier_pending_recovery_actions={}",
            apply.applied_transactions,
            apply.skipped_duplicates,
            apply.last_commit_lsn.as_deref().unwrap_or("none"),
            apply.barrier_pending_transactions,
            apply.barrier_pending_missing_manifest,
            apply.barrier_pending_missing_commit_marker,
            apply.barrier_pending_invalid_commit_marker,
            apply.barrier_pending_missing_chunks,
            apply.barrier_pending_extra_chunks,
            list_or_none(&apply.barrier_pending.blockers),
            list_or_none(&apply.barrier_pending.blocker_codes),
            list_or_none(&apply.barrier_pending.recovery_actions)
        ),
        proof_command: format!(
            "trellara status --config {config_display} --view report --format text"
        ),
    }
}

pub(crate) fn convergence_verification_run_proof(
    config_path: &Path,
    verify: Option<&VerifySummary>,
) -> RunProofGate {
    let config_display = config_path.display().to_string();
    RunProofGate {
        code: "convergence_verification".to_string(),
        status: match verify {
            Some(verify) if verify_proof_is_complete(verify) => RunProofStatus::Verified,
            Some(_) => RunProofStatus::AtRisk,
            None => RunProofStatus::NeedsEvidence,
        },
        evidence: verify
            .map(|verify| {
                format!(
                    "converged={} checksum_status={:?} source_watermark_lsn={} target_watermark_lsn={} target_caught_up={} table_count={} table_evidence={}",
                    verify.converged,
                    verify.checksum_status,
                    verify.source_watermark_lsn,
                    verify.target_watermark_lsn,
                    verify_target_caught_up(verify),
                    verify.tables.len(),
                    table_evidence(&verify.tables)
                )
            })
            .unwrap_or_else(|| "run did not include --verify; convergence proof is pending".to_string()),
        proof_command: format!("trellara verify --config {config_display}"),
    }
}

fn verify_proof_is_complete(verify: &VerifySummary) -> bool {
    verify.converged
        && verify.checksum_status == ChecksumStatus::Match
        && verify_target_caught_up(verify)
        && !verify.tables.is_empty()
        && verify.tables.iter().all(table_proof_is_complete)
}

fn verify_target_caught_up(verify: &VerifySummary) -> bool {
    ensure_target_caught_up(&verify.source_watermark_lsn, &verify.target_watermark_lsn).is_ok()
}

fn table_proof_is_complete(table: &TableVerifySummary) -> bool {
    table.converged
        && table.checksum_status == ChecksumStatus::Match
        && table.relation_match
        && table.relation == table.target_relation
        && sha256_is_valid(&table.evidence_sha256)
}

fn table_evidence(tables: &[TableVerifySummary]) -> String {
    if tables.is_empty() {
        return "none".to_string();
    }
    tables
        .iter()
        .map(table_evidence_item)
        .collect::<Vec<_>>()
        .join("; ")
}

fn table_evidence_item(table: &TableVerifySummary) -> String {
    format!(
        "relation={} target_relation={} relation_match={} converged={} checksum_status={:?} evidence_sha256={}",
        table.relation,
        table.target_relation,
        table.relation_match,
        table.converged,
        table.checksum_status,
        table.evidence_sha256
    )
}

fn sha256_is_valid(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn list_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}
