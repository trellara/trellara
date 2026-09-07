use serde::Serialize;
use trellara_checkpoint::{PartitionScaleHealthSummary, PartitionWatermarkSummary};

#[derive(Serialize)]
pub(crate) struct PartitionWatermarkReport<'a> {
    #[serde(flatten)]
    summary: &'a PartitionWatermarkSummary,
    partition_scale_health: PartitionScaleHealthSummary,
}

impl<'a> PartitionWatermarkReport<'a> {
    pub(crate) fn from_summary(summary: &'a PartitionWatermarkSummary) -> Self {
        Self {
            summary,
            partition_scale_health: PartitionScaleHealthSummary::from_watermark_summary(summary),
        }
    }
}
