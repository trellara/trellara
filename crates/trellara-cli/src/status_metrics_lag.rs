use crate::status_metric_format::{bool_value, push_metric};
use crate::{FlowStatusSummary, TargetSourceProgress};

pub(crate) fn push_checkpoint_metrics(
    output: &mut String,
    base_labels: &[(&str, &str)],
    status: &FlowStatusSummary,
) {
    push_metric(
        output,
        "trellara_source_checkpoint_exists",
        base_labels,
        bool_value(status.source_checkpoint_exists),
    );
    push_metric(
        output,
        "trellara_target_checkpoint_exists",
        base_labels,
        bool_value(status.target_checkpoint_exists),
    );
}

pub(crate) fn push_source_stream_metrics(
    output: &mut String,
    base_labels: &[(&str, &str)],
    status: &FlowStatusSummary,
) {
    push_metric(
        output,
        "trellara_source_wal_retained_bytes",
        base_labels,
        status.source_slot.retained_wal_bytes.unwrap_or_default(),
    );
    push_metric(
        output,
        "trellara_source_stream_spill_threshold_changes",
        base_labels,
        status.source_stream_spill_threshold_changes,
    );
    push_metric(
        output,
        "trellara_source_stream_spill_dir_configured",
        base_labels,
        bool_value(status.source_stream_spill_dir.is_some()),
    );
}

pub(crate) fn push_checkpoint_lag_metrics(
    output: &mut String,
    base_labels: &[(&str, &str)],
    status: &FlowStatusSummary,
) {
    if let Some(source) = status.source.as_ref() {
        push_metric(
            output,
            "trellara_source_seen_to_durable_bytes",
            base_labels,
            source.seen_to_durable_bytes,
        );
        push_metric(
            output,
            "trellara_source_seen_to_applied_bytes",
            base_labels,
            source.seen_to_applied_bytes,
        );
    }
    if let Some(target) = status.target.as_ref() {
        push_metric(
            output,
            "trellara_target_durable_to_applied_bytes",
            base_labels,
            target.durable_to_applied_bytes,
        );
    }
    if let Some(source_to_target_lag) =
        TargetSourceProgress::from_status(status).source_to_target_lag_bytes
    {
        push_metric(
            output,
            "trellara_source_to_target_lag_bytes",
            base_labels,
            source_to_target_lag,
        );
    }
}
