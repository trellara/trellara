use crate::{
    bool_value, push_metric, transaction_boundary_status_label, CorrectnessReportSummary,
    FlowStatusSummary,
};

pub(crate) fn push_transaction_boundary_metrics(
    output: &mut String,
    base_labels: &[(&str, &str)],
    status: &FlowStatusSummary,
    report: &CorrectnessReportSummary,
) {
    let boundary_status = transaction_boundary_status_label(report.transaction_boundary.status);
    let boundary_labels = [
        ("source_id", status.source_id.as_str()),
        ("dataset_id", status.dataset_id.as_str()),
        ("mode", status.mode.as_str()),
        ("status", boundary_status),
    ];
    push_metric(
        output,
        "trellara_transaction_boundary_status",
        &boundary_labels,
        1,
    );
    push_metric(
        output,
        "trellara_transaction_boundary_source_checkpoint_durable",
        base_labels,
        bool_value(report.transaction_boundary.source_checkpoint_durable),
    );
    push_metric(
        output,
        "trellara_transaction_boundary_target_checkpoint_caught_up",
        base_labels,
        bool_value(report.transaction_boundary.target_checkpoint_caught_up),
    );
    push_metric(
        output,
        "trellara_transaction_boundary_manifest_barrier_required",
        base_labels,
        bool_value(report.transaction_boundary.manifest_barrier_required),
    );
    if let Some(complete) = report.transaction_boundary.manifest_barrier_complete {
        push_metric(
            output,
            "trellara_transaction_boundary_manifest_barrier_complete",
            base_labels,
            bool_value(complete),
        );
    }
    if let Some(caught_up) = report
        .transaction_boundary
        .global_partition_watermark_caught_up
    {
        push_metric(
            output,
            "trellara_transaction_boundary_global_partition_watermark_caught_up",
            base_labels,
            bool_value(caught_up),
        );
    }
}

pub(crate) fn push_partition_watermark_metrics(
    output: &mut String,
    base_labels: &[(&str, &str)],
    status: &FlowStatusSummary,
) {
    if let Some(watermarks) = status.partition_watermarks.as_ref() {
        push_metric(
            output,
            "trellara_partition_watermark_complete",
            base_labels,
            bool_value(watermarks.complete_partition_set),
        );
        push_metric(
            output,
            "trellara_partition_watermark_missing_partitions_total",
            base_labels,
            watermarks.missing_partitions.len(),
        );
        push_metric(
            output,
            "trellara_partition_global_durable_to_applied_bytes",
            base_labels,
            watermarks
                .global_durable_to_applied_bytes
                .unwrap_or_default(),
        );
    }
}
