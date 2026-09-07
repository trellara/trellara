use std::path::Path;

use crate::{
    fleet_lake_fanin_recovery_drills, DatasetMode, FleetRecoveryDrill, StreamConfig, TrellaraConfig,
};

pub(crate) fn fleet_recovery_drills(
    config: &TrellaraConfig,
    path: &Path,
) -> Vec<FleetRecoveryDrill> {
    let config_display = path.display();
    let mut target_quarantine_commands = vec![
        format!("trellara status --config {config_display} --view alerts --format text"),
        format!("trellara repair-plan --config {config_display}"),
        format!(
            "trellara quarantine replay-ready --config {config_display} --transaction-id <tx> --commit-lsn <lsn>"
        ),
    ];
    if let Some(command) = recovery_locate_command(config, path) {
        target_quarantine_commands.push(command);
    }
    target_quarantine_commands.push(recovery_redelivery_command(config, path));
    target_quarantine_commands.push(format!("trellara verify --config {config_display}"));
    let mut drills = vec![
        FleetRecoveryDrill {
            code: "target_quarantine_replay".to_string(),
            trigger: "target apply fails closed into trellara.apply_quarantine".to_string(),
            operator_goal: "repair the target contract, mark the transaction replay-ready, and redeliver without advancing checkpoints prematurely".to_string(),
            commands: target_quarantine_commands,
            success_evidence: "status report shows no target quarantine, target checkpoint advances after replay, and verify converges with checksum_status=match".to_string(),
        },
        FleetRecoveryDrill {
            code: "checksum_reseed".to_string(),
            trigger: "verify reports row-count drift or checksum mismatch".to_string(),
            operator_goal: "reseed the affected scope from a fresh snapshot handoff and prove convergence before resuming trust".to_string(),
            commands: vec![
                format!("trellara verify --config {config_display}"),
                format!("trellara reseed --config {config_display} --table <schema.table>"),
                format!("trellara status --config {config_display} --view report --format text"),
                format!("trellara verify --config {config_display} --table <schema.table>"),
            ],
            success_evidence: "latest reseed handoff watermark is visible in status and table-scoped verify converges".to_string(),
        },
        FleetRecoveryDrill {
            code: "source_wal_loss_reseed".to_string(),
            trigger: "source slot is invalidated, missing, or beyond WAL retention".to_string(),
            operator_goal: "recreate capture from a fresh audited handoff and avoid replay gaps after retained WAL is gone".to_string(),
            commands: vec![
                format!("trellara check --config {config_display} --format text"),
                format!("trellara reseed --config {config_display}"),
                format!("trellara snapshot --config {config_display} --force"),
                format!("trellara status --config {config_display} --view report --format text"),
                format!("trellara verify --config {config_display}"),
            ],
            success_evidence: "source safety is clear, snapshot handoff is fresh, and verify converges after the recreated slot resumes".to_string(),
        },
        FleetRecoveryDrill {
            code: "schema_handoff_refresh".to_string(),
            trigger: "pinned source schema fingerprint drifts or target compatibility changes".to_string(),
            operator_goal: "refresh the contract, perform a fresh handoff, and verify before allowing CDC to continue".to_string(),
            commands: vec![
                format!("trellara schema-discover --config {config_display}"),
                format!("trellara contract-test --config {config_display}"),
                format!("trellara snapshot --config {config_display} --force"),
                format!("trellara verify --config {config_display}"),
            ],
            success_evidence: "contract-test passes with updated fingerprints and status reports a fresh audited snapshot handoff".to_string(),
        },
    ];

    if config.dataset.mode == DatasetMode::PartitionedScaleMode {
        drills.push(FleetRecoveryDrill {
            code: "partition_barrier_replay".to_string(),
            trigger: "partitioned manifest, commit marker, or one partition lane is missing during replay".to_string(),
            operator_goal: "replay the complete manifest, commit, and partition topic set before exposing global visibility".to_string(),
            commands: vec![
                format!("trellara partition-local --config {config_display}"),
                format!("trellara partition-watermarks --config {config_display}"),
            ]
            .into_iter()
            .chain(recovery_locate_command(config, path))
            .chain(std::iter::once(recovery_redelivery_command(config, path)))
            .chain(std::iter::once(format!(
                "trellara status --config {config_display} --view report --format text"
            )))
            .collect(),
            success_evidence: "partition watermarks are complete and global visibility advances only after the manifest barrier is reconstructed".to_string(),
        });
    }

    drills.extend(fleet_lake_fanin_recovery_drills(config, path));

    drills
}

fn recovery_locate_command(config: &TrellaraConfig, path: &Path) -> Option<String> {
    let config_display = path.display();
    if matches!(config.stream, StreamConfig::Local { .. }) {
        Some(format!(
            "trellara stream locate-local --config {config_display} --transaction-id <tx> --commit-lsn <lsn>"
        ))
    } else {
        None
    }
}

fn recovery_redelivery_command(config: &TrellaraConfig, path: &Path) -> String {
    let config_display = path.display();
    if matches!(config.stream, StreamConfig::Local { .. }) {
        format!("trellara stream seek-local --config {config_display} --topic <topic> --next-offset <offset>")
    } else {
        format!("redeliver or seek Kafka topics returned by trellara quarantine replay-ready --config {config_display} --transaction-id <tx> --commit-lsn <lsn>")
    }
}
