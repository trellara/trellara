use clap::Parser;

use crate::{NativeWorkerReportStatus, QuickstartOutputFormat};

#[derive(Clone, Debug, Parser)]
pub struct NativeWorkerReportArgs {
    #[arg(long, value_enum)]
    pub status: NativeWorkerReportStatus,
    #[arg(long, default_value_t = 0)]
    pub drained_frames: usize,
    #[arg(long)]
    pub source_feedback_lsn: Option<String>,
    #[arg(long, default_value = "operator_supplied_native_worker_report")]
    pub reason: String,
    #[arg(long, value_enum, default_value_t = QuickstartOutputFormat::Json)]
    pub format: QuickstartOutputFormat,
}
