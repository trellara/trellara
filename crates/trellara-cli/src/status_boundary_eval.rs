use crate::{
    status_boundary::TransactionBoundaryStatus,
    status_boundary_modes::{
        is_manifest_barrier_mode, is_partitioned_mode, is_strict_chunked_mode,
    },
    FlowStatusSummary, TargetSourceProgress,
};

pub(crate) struct BoundaryEvaluation {
    pub(crate) status: TransactionBoundaryStatus,
    pub(crate) source_checkpoint_durable: bool,
    pub(crate) target_checkpoint_caught_up: bool,
    pub(crate) manifest_barrier_required: bool,
    pub(crate) manifest_barrier_complete: Option<bool>,
    pub(crate) global_partition_watermark_caught_up: Option<bool>,
}

pub(crate) fn evaluate_transaction_boundary(status: &FlowStatusSummary) -> BoundaryEvaluation {
    let source_checkpoint_durable = status
        .source
        .as_ref()
        .map(|source| source.source_is_durable)
        .unwrap_or(false);
    let target_checkpoint_caught_up =
        TargetSourceProgress::from_status(status).reaches_source_durable;
    let is_partitioned = is_partitioned_mode(&status.mode);
    let is_strict_chunked = is_strict_chunked_mode(&status.mode);
    let manifest_barrier_required = is_manifest_barrier_mode(&status.mode);
    let manifest_barrier_complete = manifest_barrier_complete(
        status,
        is_partitioned,
        is_strict_chunked,
        target_checkpoint_caught_up,
    );
    let global_partition_watermark_caught_up =
        global_partition_watermark_caught_up(status, is_partitioned);
    let boundary_is_clean = source_checkpoint_durable
        && target_checkpoint_caught_up
        && status.latest_quarantine.is_none()
        && barrier_is_clean(
            is_partitioned,
            is_strict_chunked,
            manifest_barrier_complete,
            global_partition_watermark_caught_up,
        );

    BoundaryEvaluation {
        status: boundary_status(status, is_partitioned, boundary_is_clean),
        source_checkpoint_durable,
        target_checkpoint_caught_up,
        manifest_barrier_required,
        manifest_barrier_complete,
        global_partition_watermark_caught_up,
    }
}

fn manifest_barrier_complete(
    status: &FlowStatusSummary,
    is_partitioned: bool,
    is_strict_chunked: bool,
    target_checkpoint_caught_up: bool,
) -> Option<bool> {
    if is_partitioned {
        status
            .partition_watermarks
            .as_ref()
            .map(|watermarks| watermarks.complete_partition_set)
    } else if is_strict_chunked && status.target.is_some() {
        Some(target_checkpoint_caught_up && status.latest_quarantine.is_none())
    } else {
        None
    }
}

fn global_partition_watermark_caught_up(
    status: &FlowStatusSummary,
    is_partitioned: bool,
) -> Option<bool> {
    if is_partitioned {
        status.partition_watermarks.as_ref().map(|watermarks| {
            watermarks
                .global_durable_to_applied_bytes
                .is_some_and(|lag| lag == 0)
        })
    } else {
        None
    }
}

fn barrier_is_clean(
    is_partitioned: bool,
    is_strict_chunked: bool,
    manifest_barrier_complete: Option<bool>,
    global_partition_watermark_caught_up: Option<bool>,
) -> bool {
    if is_partitioned {
        manifest_barrier_complete == Some(true)
            && global_partition_watermark_caught_up == Some(true)
    } else if is_strict_chunked {
        manifest_barrier_complete == Some(true)
    } else {
        true
    }
}

fn boundary_status(
    status: &FlowStatusSummary,
    is_partitioned: bool,
    boundary_is_clean: bool,
) -> TransactionBoundaryStatus {
    if boundary_is_clean {
        TransactionBoundaryStatus::Verified
    } else if has_required_evidence(status, is_partitioned) {
        TransactionBoundaryStatus::AtRisk
    } else {
        TransactionBoundaryStatus::PendingEvidence
    }
}

fn has_required_evidence(status: &FlowStatusSummary, is_partitioned: bool) -> bool {
    status.source.is_some()
        && status.target.is_some()
        && (!is_partitioned || status.partition_watermarks.is_some())
}
