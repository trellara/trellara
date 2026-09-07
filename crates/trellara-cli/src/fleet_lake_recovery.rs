use std::path::Path;

use crate::{DatasetMode, FleetRecoveryDrill, TrellaraConfig};

pub(crate) fn fleet_lake_fanin_recovery_drills(
    config: &TrellaraConfig,
    path: &Path,
) -> Vec<FleetRecoveryDrill> {
    let config_display = path.display();
    let mut drills = vec![
        FleetRecoveryDrill {
            code: "lake_offline_source_gap_acceptance".to_string(),
            trigger: "one or more required fleet sources are offline when a lake epoch seals"
                .to_string(),
            operator_goal: "publish only an explicit complete-with-gaps epoch, name every missing source, and block default Spark consumption unless the customer accepts the gap".to_string(),
            commands: vec![
                format!(
                    "trellara lake epoch --config {config_display} --scenario offline-stores-publish-with-gaps --format text"
                ),
                format!(
                    "trellara lake fanin verify --config {config_display} --stream-epoch <stream-epoch.json> --lake-epoch <lake-epoch.json> --format text"
                ),
                format!(
                    "trellara lake spark-template current-state --config {config_display} --table <schema.table> --epoch-id <epoch-id> --accept-complete-with-gaps"
                ),
            ],
            success_evidence: "lake epoch reports complete_with_gaps with source_watermarks naming missing sources, verification is match, and Spark jobs require explicit gap acceptance".to_string(),
        },
        FleetRecoveryDrill {
            code: "lake_late_source_recompute".to_string(),
            trigger: "a missing source returns after an epoch was published with gaps".to_string(),
            operator_goal: "ingest the late source, recompute the epoch to complete, and republish Spark-derived tables from the completed epoch".to_string(),
            commands: vec![
                format!(
                    "trellara lake epoch --config {config_display} --scenario late-store-recovery-completes-epoch --format text"
                ),
                format!(
                    "trellara lake fanin verify --config {config_display} --stream-epoch <stream-epoch.json> --lake-epoch <lake-epoch.json> --format text"
                ),
                format!(
                    "trellara lake spark-template scd2 --config {config_display} --table <schema.table> --epoch-id <epoch-id>"
                ),
            ],
            success_evidence: "lake epoch transitions from complete_with_gaps to complete, missing_source_count is zero, and verification remains match before derived tables are exposed".to_string(),
        },
        FleetRecoveryDrill {
            code: "lake_conflicting_duplicate_quarantine".to_string(),
            trigger: "duplicate replay carries conflicting idempotency evidence for a source transaction".to_string(),
            operator_goal: "quarantine the source evidence, prevent ambiguous current-state/SCD2 publication, and require repair or reseed before consumption".to_string(),
            commands: vec![
                format!(
                    "trellara lake epoch --config {config_display} --scenario conflicting-duplicate-quarantine --format text"
                ),
                format!(
                    "trellara lake fanin verify --config {config_display} --stream-epoch <stream-epoch.json> --lake-epoch <lake-epoch.json> --format text"
                ),
                format!("trellara reseed --config {config_display} --table <schema.table>"),
            ],
            success_evidence: "lake epoch reports quarantined source_watermarks with conflicting duplicate gap_reason and Spark consumption stays blocked until verification is repaired".to_string(),
        },
    ];

    if config.dataset.mode == DatasetMode::PartitionedScaleMode {
        drills.push(FleetRecoveryDrill {
            code: "lake_missing_manifest_chunk_replay".to_string(),
            trigger: "partitioned fan-in detects a missing manifest chunk or incomplete partition lane before epoch publication".to_string(),
            operator_goal: "reconstruct the manifest and partition watermarks before any global lake epoch visibility is claimed".to_string(),
            commands: vec![
                format!("trellara partition-watermarks --config {config_display} --format text"),
                format!(
                    "trellara lake epoch --config {config_display} --scenario offline-stores-publish-with-gaps --format text"
                ),
                format!(
                    "trellara lake fanin verify --config {config_display} --stream-epoch <stream-epoch.json> --lake-epoch <lake-epoch.json> --format text"
                ),
            ],
            success_evidence: "partition watermarks show a complete partition set before lake epoch source_watermarks advance to the global low watermark".to_string(),
        });
    }

    drills
}
