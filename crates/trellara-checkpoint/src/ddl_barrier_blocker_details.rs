use crate::ddl_barrier_blockers::DdlBarrierBlockerContext;
use crate::{DdlBarrierReleaseBlocker, DdlBarrierSinkEvidence};

pub(crate) fn ddl_release_blocker_details(
    barrier: DdlBarrierBlockerContext<'_>,
    pending_sinks: &[String],
    rejected_sinks: &[String],
    unexpected_sinks: &[String],
    sink_evidence: &[DdlBarrierSinkEvidence],
) -> Vec<DdlBarrierReleaseBlocker> {
    let context = BlockerEvidenceContext {
        barrier_id: barrier.barrier_id,
        barrier_lsn: barrier.barrier_lsn,
        schema_version: barrier.schema_version,
    };
    let mut blockers = Vec::new();
    if !barrier.has_required_sinks {
        blockers.push(DdlBarrierReleaseBlocker {
            code: "missing_required_sinks".to_string(),
            message: "missing required sink acknowledgements contract".to_string(),
            sinks: Vec::new(),
            evidence: format!(
                "barrier_id={} barrier_lsn={} schema_version={} required_sinks=none",
                barrier.barrier_id, barrier.barrier_lsn, barrier.schema_version
            ),
        });
    }
    if !pending_sinks.is_empty() {
        blockers.push(DdlBarrierReleaseBlocker {
            code: "pending_required_ack".to_string(),
            message: "pending required sink acknowledgements".to_string(),
            sinks: pending_sinks.to_vec(),
            evidence: context.with_sinks("pending_sinks", pending_sinks),
        });
    }
    if barrier.requires_global_partition_pause
        && pending_sinks
            .iter()
            .any(|sink| sink == "partition_visibility")
    {
        blockers.push(DdlBarrierReleaseBlocker {
            code: "partition_visibility_not_released".to_string(),
            message: "partition visibility watermark has not released post-DDL DML".to_string(),
            sinks: vec!["partition_visibility".to_string()],
            evidence: format!(
                "barrier_id={} barrier_lsn={} schema_version={} required_sink=partition_visibility",
                barrier.barrier_id, barrier.barrier_lsn, barrier.schema_version
            ),
        });
    }
    if !rejected_sinks.is_empty() {
        blockers.push(DdlBarrierReleaseBlocker {
            code: "rejected_or_stale_ack".to_string(),
            message: "rejected or stale sink acknowledgements".to_string(),
            sinks: rejected_sinks.to_vec(),
            evidence: context.with_rejected_sinks(rejected_sinks, sink_evidence),
        });
    }
    if !unexpected_sinks.is_empty() {
        blockers.push(DdlBarrierReleaseBlocker {
            code: "unexpected_ack".to_string(),
            message: "unexpected sink acknowledgements".to_string(),
            sinks: unexpected_sinks.to_vec(),
            evidence: context.with_sinks("unexpected_sinks", unexpected_sinks),
        });
    }
    blockers
}

struct BlockerEvidenceContext<'a> {
    barrier_id: &'a str,
    barrier_lsn: &'a str,
    schema_version: &'a str,
}

impl BlockerEvidenceContext<'_> {
    fn with_sinks(&self, label: &str, sinks: &[String]) -> String {
        format!(
            "barrier_id={} barrier_lsn={} schema_version={} {}={}",
            self.barrier_id,
            self.barrier_lsn,
            self.schema_version,
            label,
            sinks.join(", ")
        )
    }

    fn with_rejected_sinks(
        &self,
        rejected_sinks: &[String],
        sink_evidence: &[DdlBarrierSinkEvidence],
    ) -> String {
        format!(
            "{} rejection_codes={}",
            self.with_sinks("rejected_sinks", rejected_sinks),
            rejection_codes(rejected_sinks, sink_evidence)
        )
    }
}

fn rejection_codes(rejected_sinks: &[String], sink_evidence: &[DdlBarrierSinkEvidence]) -> String {
    let codes = rejected_sinks
        .iter()
        .map(|sink| format!("{sink}:{}", rejection_code_for_sink(sink, sink_evidence)))
        .collect::<Vec<_>>();
    codes.join(", ")
}

fn rejection_code_for_sink(sink: &str, sink_evidence: &[DdlBarrierSinkEvidence]) -> String {
    sink_evidence
        .iter()
        .find(|evidence| evidence.sink == sink)
        .and_then(|evidence| evidence.rejection_code.as_deref())
        .unwrap_or("unknown")
        .to_string()
}
