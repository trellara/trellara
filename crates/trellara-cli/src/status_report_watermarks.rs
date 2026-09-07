use crate::FlowStatusSummary;

pub(crate) struct CorrectnessReportWatermarks {
    pub(crate) source_lsn: Option<String>,
    pub(crate) target_lsn: Option<String>,
    pub(crate) partition_global_applied_lsn: Option<String>,
    pub(crate) reseed_lsn: Option<String>,
    pub(crate) snapshot_handoff_watermark_lsn: Option<String>,
    pub(crate) snapshot_handoff_relation: Option<String>,
    pub(crate) snapshot_run_id: Option<String>,
    pub(crate) snapshot_state: Option<String>,
    pub(crate) snapshot_consistent_lsn: Option<String>,
}

impl CorrectnessReportWatermarks {
    pub(crate) fn from_status(status: &FlowStatusSummary) -> Self {
        Self {
            source_lsn: status
                .source
                .as_ref()
                .map(|source| source.last_durable_lsn.clone()),
            target_lsn: status
                .target
                .as_ref()
                .map(|target| target.last_applied_lsn.clone()),
            partition_global_applied_lsn: status
                .partition_watermarks
                .as_ref()
                .and_then(|watermarks| watermarks.global_applied_lsn.clone()),
            reseed_lsn: status
                .latest_reseed
                .as_ref()
                .map(|reseed| reseed.watermark_lsn.clone()),
            snapshot_handoff_watermark_lsn: status
                .latest_snapshot_handoff
                .as_ref()
                .map(|handoff| handoff.watermark_lsn.clone()),
            snapshot_handoff_relation: status
                .latest_snapshot_handoff
                .as_ref()
                .map(|handoff| handoff.relation.clone()),
            snapshot_run_id: status
                .latest_snapshot_run
                .as_ref()
                .map(|run| run.run_id.clone()),
            snapshot_state: status
                .latest_snapshot_run
                .as_ref()
                .map(|run| run.state.to_string()),
            snapshot_consistent_lsn: status
                .latest_snapshot_run
                .as_ref()
                .and_then(|run| run.consistent_lsn.clone()),
        }
    }
}
