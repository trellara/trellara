use trellara_checkpoint::PartitionWatermarkSummary;

use crate::{
    status_partition_report::PartitionWatermarkReport,
    status_partition_text::render_partition_watermark_text, PilotGuideOutputFormat, Result,
};

pub(crate) fn render_partition_watermark_summary(
    summary: &PartitionWatermarkSummary,
    format: PilotGuideOutputFormat,
) -> Result<String> {
    match format {
        PilotGuideOutputFormat::Json => Ok(serde_json::to_string_pretty(
            &PartitionWatermarkReport::from_summary(summary),
        )?),
        PilotGuideOutputFormat::Text => Ok(render_partition_watermark_text(summary)),
    }
}
