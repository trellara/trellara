use trellara_checkpoint::DdlBarrierSummary;

use crate::list_or_none;

pub(super) fn ddl_release_evidence(summary: &DdlBarrierSummary) -> Vec<String> {
    let mut evidence = vec![
        format!(
            "barrier {} recorded at lsn {} with schema_version {}",
            summary.barrier_id, summary.barrier_lsn, summary.schema_version
        ),
        format!(
            "cdc_transaction_boundary: {}",
            summary.cdc_transaction_boundary
        ),
        format!(
            "{}/{} required sink ACKs accepted",
            summary.acked_sink_count, summary.required_sink_count
        ),
        format!(
            "post-DDL DML release_dml={} blocker_codes={}",
            summary.release_dml,
            list_or_none(&summary.release_blocker_codes)
        ),
    ];
    if summary.requires_global_partition_pause {
        evidence.push(partition_visibility_release_evidence(summary));
    }
    evidence
}

fn partition_visibility_release_evidence(summary: &DdlBarrierSummary) -> String {
    summary
        .sink_evidence
        .iter()
        .find(|evidence| evidence.sink == "partition_visibility")
        .and_then(|evidence| evidence.detail.as_ref())
        .map(|detail| {
            format!(
                "partition_visibility release evidence for {}: {detail}",
                summary.barrier_id
            )
        })
        .unwrap_or_else(|| {
            format!(
                "partition_visibility release evidence missing for {} at barrier_lsn {}",
                summary.barrier_id, summary.barrier_lsn
            )
        })
}
