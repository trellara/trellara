use crate::raw_cdc_types::{LakeRawCdcCommitStep, LakeRawCdcDataFilePlan};

pub(super) fn commit_steps(
    data_files: &[LakeRawCdcDataFilePlan],
    epoch_id: &str,
) -> Vec<LakeRawCdcCommitStep> {
    let mut steps: Vec<LakeRawCdcCommitStep> = data_files
        .iter()
        .enumerate()
        .map(|(index, file)| raw_data_step(index, file, epoch_id))
        .collect();
    steps.extend(metadata_commit_steps(epoch_id));
    steps
}

fn raw_data_step(
    index: usize,
    file: &LakeRawCdcDataFilePlan,
    epoch_id: &str,
) -> LakeRawCdcCommitStep {
    LakeRawCdcCommitStep {
        order: 10 + index as u32,
        phase: "raw_cdc_data".to_string(),
        action: format!(
            "append {} changes for {} to {}",
            file.change_count, file.relation, file.object_key_hint
        ),
        durability_gate: "object exists and file checksum is durable before source acknowledgement or epoch metadata publication".to_string(),
        recovery_rule: format!(
            "if the writer crashes before epoch {epoch_id} metadata is visible, replay this data-file intent idempotently by Trellara idempotency keys"
        ),
    }
}

fn metadata_commit_steps(epoch_id: &str) -> Vec<LakeRawCdcCommitStep> {
    [
        (
            100,
            "epoch_sources_metadata",
            "publish per-source watermarks and row counts",
        ),
        (
            110,
            "epoch_tables_metadata",
            "publish per-table transaction and change rollups",
        ),
        (
            120,
            "epoch_partitions_metadata",
            "publish per-source partition transaction and event rollups",
        ),
        (130, "epoch_row_metadata", "publish consumable epoch row"),
        (
            140,
            "verification_metadata",
            "publish stream-to-lake verification row",
        ),
    ]
    .into_iter()
    .map(|(order, phase, action)| LakeRawCdcCommitStep {
        order,
        phase: phase.to_string(),
        action: format!("{action} for epoch {epoch_id}"),
        durability_gate: "all planned raw CDC data files and Iceberg checkpoint receipts for the epoch are durable before this metadata step".to_string(),
        recovery_rule: format!(
            "if the writer crashes after this step, verify checkpoint receipts and discover epoch {epoch_id} metadata before publishing duplicate visibility"
        ),
    })
    .collect()
}
