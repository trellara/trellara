use crate::strict_chunk::{
    StrictChunkSimulationConfig, StrictChunkSimulationReport, StrictChunkSimulationStep,
};

pub(crate) struct StrictChunkReportInput {
    pub(crate) config: StrictChunkSimulationConfig,
    pub(crate) transaction_id: String,
    pub(crate) commit_lsn: u64,
    pub(crate) chunks_published: usize,
    pub(crate) duplicate_chunks: usize,
    pub(crate) manifest_published: bool,
    pub(crate) source_acknowledged_lsn: Option<u64>,
    pub(crate) target_applied_lsn: Option<u64>,
    pub(crate) applied_transactions: usize,
    pub(crate) injected_failure: Option<String>,
    pub(crate) steps: Vec<StrictChunkSimulationStep>,
}

pub(crate) fn build_strict_chunk_report(
    input: StrictChunkReportInput,
) -> StrictChunkSimulationReport {
    let passed = report_passed(&input);

    StrictChunkSimulationReport {
        seed: input.config.seed,
        failure_point: input.config.failure_point,
        transaction_id: input.transaction_id,
        commit_lsn: input.commit_lsn,
        chunk_count: input.config.chunk_count,
        chunks_published: input.chunks_published,
        duplicate_chunks: input.duplicate_chunks,
        manifest_published: input.manifest_published,
        source_acknowledged_lsn: input.source_acknowledged_lsn,
        target_applied_lsn: input.target_applied_lsn,
        applied_transactions: input.applied_transactions,
        passed,
        injected_failure: input.injected_failure,
        steps: input.steps,
    }
}

fn report_passed(input: &StrictChunkReportInput) -> bool {
    input.injected_failure.is_some()
        && input.manifest_published
        && input.chunks_published == input.config.chunk_count
        && input.applied_transactions == 1
        && input.target_applied_lsn == Some(input.commit_lsn)
        && input.source_acknowledged_lsn == Some(input.commit_lsn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strict_chunk::StrictChunkFailurePoint;

    fn passing_input() -> StrictChunkReportInput {
        StrictChunkReportInput {
            config: StrictChunkSimulationConfig::new(
                11,
                StrictChunkFailurePoint::RelayCrashAfterChunksBeforeManifest,
            ),
            transaction_id: "tx-1".to_string(),
            commit_lsn: 42,
            chunks_published: 4,
            duplicate_chunks: 1,
            manifest_published: true,
            source_acknowledged_lsn: Some(42),
            target_applied_lsn: Some(42),
            applied_transactions: 1,
            injected_failure: Some("relay crash".to_string()),
            steps: Vec::new(),
        }
    }

    #[test]
    fn strict_chunk_report_passes_after_complete_manifest_bounded_apply() {
        let report = build_strict_chunk_report(passing_input());

        assert!(report.passed);
        assert_eq!(report.chunk_count, report.chunks_published);
        assert_eq!(report.source_acknowledged_lsn, report.target_applied_lsn);
    }

    #[test]
    fn strict_chunk_report_fails_without_manifest_visibility_boundary() {
        let mut input = passing_input();
        input.manifest_published = false;

        assert!(!build_strict_chunk_report(input).passed);
    }

    #[test]
    fn strict_chunk_report_fails_when_source_ack_precedes_target_apply() {
        let mut input = passing_input();
        input.target_applied_lsn = None;
        input.applied_transactions = 0;

        assert!(!build_strict_chunk_report(input).passed);
    }
}
