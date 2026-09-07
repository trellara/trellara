use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::{CliError, PilotPackageArtifact, Result, TrellaraConfig};

pub(crate) fn write_pilot_package_artifact(
    output: &Path,
    file_name: &str,
    kind: &str,
    purpose: &str,
    contents: &str,
) -> Result<PilotPackageArtifact> {
    let path = output.join(file_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| CliError::WriteOutput {
            path: parent.display().to_string(),
            source,
        })?;
    }
    fs::write(&path, contents).map_err(|source| CliError::WriteOutput {
        path: path.display().to_string(),
        source,
    })?;
    let byte_count = contents.len();
    let sha256 = format!("{:x}", Sha256::digest(contents.as_bytes()));

    Ok(PilotPackageArtifact {
        path: path.display().to_string(),
        kind: kind.to_string(),
        purpose: purpose.to_string(),
        byte_count,
        sha256,
    })
}

pub(crate) fn write_pilot_package_binary_artifact(
    output: &Path,
    file_name: &str,
    kind: &str,
    purpose: &str,
    contents: &[u8],
) -> Result<PilotPackageArtifact> {
    let path = output.join(file_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| CliError::WriteOutput {
            path: parent.display().to_string(),
            source,
        })?;
    }
    fs::write(&path, contents).map_err(|source| CliError::WriteOutput {
        path: path.display().to_string(),
        source,
    })?;
    let byte_count = contents.len();
    let sha256 = format!("{:x}", Sha256::digest(contents));

    Ok(PilotPackageArtifact {
        path: path.display().to_string(),
        kind: kind.to_string(),
        purpose: purpose.to_string(),
        byte_count,
        sha256,
    })
}

pub(crate) fn render_local_run_proof_artifact(
    config: &TrellaraConfig,
    config_path: &Path,
) -> String {
    let config_display = config_path.display();
    let mut output = String::new();
    writeln!(&mut output, "# Trellara Local Run Proof").expect("write string");
    writeln!(&mut output).expect("write string");
    writeln!(&mut output, "config: {config_display}").expect("write string");
    writeln!(&mut output, "source: {}", config.source.id).expect("write string");
    writeln!(&mut output, "dataset: {}", config.dataset.id).expect("write string");
    writeln!(&mut output, "mode: {}", config.status_mode()).expect("write string");
    writeln!(&mut output).expect("write string");
    output.push_str("## Command\n\n");
    writeln!(
        &mut output,
        "```sh\ntrellara run --local --verify --format text --config {config_display} --snapshot-run-id local-run-snapshot --max-transactions 100 --max-messages 100\n```"
    )
    .expect("write string");
    output.push_str("\n## Evidence To Retain\n\n");
    output.push_str("- `phase_summary`: confirms bootstrap, snapshot, relay, apply, and verify phases ran under one local evaluation loop.\n");
    output.push_str("- `snapshot_handoff_boundary`: proves the initial snapshot reached a stream handoff or verified state at a named consistent LSN with `selected_table_count` matching completed table evidence.\n");
    output.push_str("- `snapshot_handoff_blocker_codes`: stable snapshot blockers such as missing table progress, incomplete copy, missing watermark, or mismatched handoff LSN.\n");
    output.push_str("- `snapshot_handoff_recovery_actions`: concrete resume, fresh handoff, or evidence repair guidance before CDC catch-up can be trusted.\n");
    output.push_str("- `source_bootstrap_position`: records publication, slot, relation count, preflight posture, and consistent LSN.\n");
    output.push_str("- `bounded_large_transaction_capture`: proves pgoutput v2 streaming, spill threshold, `bounded_memory_contract`, and `visibility_contract` evidence so large transactions stay bounded while commit visibility waits for the configured manifest barrier.\n");
    output.push_str("- `source_ack_after_local_durability`: requires `source_ack_lsn`, `source_ack_durability_proof`, proof that every Trellara publish ack is durable, and source ACK publish destinations matched to the planned Trellara stream before source acknowledgement advances.\n");
    output.push_str("- `target_apply_checkpoint`: proves target apply, dedup, checkpoint evidence, and `barrier_pending_blockers` advanced together.\n");
    output.push_str("- `barrier_pending_blockers`: must be `none` before accepting a manifest-barrier transaction as fully visible; missing manifest, commit marker, missing chunks, or extra chunks stay explicit.\n");
    output.push_str("- `barrier_pending_blocker_codes`: stable blocker codes (`missing_manifest`, `missing_commit_marker`, `invalid_commit_marker`, `missing_chunks`, `extra_chunks`) for automation and support triage.\n");
    output.push_str("- `barrier_pending_recovery_actions`: stable operator guidance for replay, republish, quarantine, or rewind decisions before target apply continues.\n");
    output.push_str("- `convergence_verification`: proves row-count, checksum, and target relation identity convergence when `--verify` is included.\n");
    output.push_str("\n## Review Rule\n\n");
    output.push_str("Treat `verified` gates as evidence, `needs_evidence` gates as pilot follow-up, and `at_risk` gates as blockers until the linked proof command is clean.\n");
    output
}
