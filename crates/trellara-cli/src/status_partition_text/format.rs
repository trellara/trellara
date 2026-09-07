use trellara_checkpoint::PartitionScaleHealthStatus;

pub(super) fn partition_scale_health_status_label(
    status: &PartitionScaleHealthStatus,
) -> &'static str {
    match status {
        PartitionScaleHealthStatus::MissingPartitions => "missing_partitions",
        PartitionScaleHealthStatus::LaggingPartitions => "lagging_partitions",
        PartitionScaleHealthStatus::Ready => "ready",
    }
}

pub(super) fn option_u64(value: &Option<u64>) -> String {
    value
        .map(|number| number.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

pub(super) fn csv_u32(values: &[u32]) -> String {
    if values.is_empty() {
        return "none".to_string();
    }

    values
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

pub(super) fn csv_str(values: &[String]) -> String {
    if values.is_empty() {
        return "none".to_string();
    }

    values.join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partition_health_labels_match_text_surface() {
        assert_eq!(
            partition_scale_health_status_label(&PartitionScaleHealthStatus::MissingPartitions),
            "missing_partitions"
        );
        assert_eq!(
            partition_scale_health_status_label(&PartitionScaleHealthStatus::LaggingPartitions),
            "lagging_partitions"
        );
        assert_eq!(
            partition_scale_health_status_label(&PartitionScaleHealthStatus::Ready),
            "ready"
        );
    }

    #[test]
    fn empty_partition_lists_render_as_none() {
        assert_eq!(csv_u32(&[]), "none");
        assert_eq!(csv_u32(&[0, 2]), "0,2");
    }

    #[test]
    fn empty_string_lists_render_as_none() {
        assert_eq!(csv_str(&[]), "none");
        assert_eq!(
            csv_str(&["missing_partition_watermarks".to_string()]),
            "missing_partition_watermarks"
        );
    }

    #[test]
    fn absent_numeric_values_render_as_unknown() {
        assert_eq!(option_u64(&None), "unknown");
        assert_eq!(option_u64(&Some(42)), "42");
    }
}
