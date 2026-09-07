use std::path::PathBuf;

use clap::Parser;

use crate::{PilotGuideOutputFormat, QuickstartOutputFormat, StatusView};

#[derive(Clone, Debug, Parser)]
pub struct ConfigArgs {
    #[arg(short, long)]
    pub config: PathBuf,
}

#[derive(Clone, Debug, Parser)]
pub struct PartitionWatermarksArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long, value_enum, default_value_t = PilotGuideOutputFormat::Json)]
    pub format: PilotGuideOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct PartitionRebalancePlanArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long, value_enum, default_value_t = PilotGuideOutputFormat::Json)]
    pub format: PilotGuideOutputFormat,
    #[arg(long, default_value_t = 50)]
    pub max_skew_percent: u32,
    #[arg(long)]
    pub plan_moves: bool,
    #[arg(long = "partition-event-count", value_name = "PARTITION=COUNT")]
    pub partition_event_counts: Vec<String>,
}

#[derive(Clone, Debug, Parser)]
pub struct StatusArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long, value_enum, default_value_t = StatusView::Flow)]
    pub view: StatusView,
    #[arg(long, value_enum, default_value_t = QuickstartOutputFormat::Json)]
    pub format: QuickstartOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct QuickstartArgs {
    #[arg(short, long, default_value = "trellara.yml")]
    pub config: PathBuf,
    #[arg(long)]
    pub check: bool,
    #[arg(long, value_enum, default_value_t = QuickstartOutputFormat::Json)]
    pub format: QuickstartOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct MvpCheckArgs {
    #[arg(short, long, default_value = "examples/retail-fleet/local.yml")]
    pub config: PathBuf,
    #[arg(long, value_enum, default_value_t = QuickstartOutputFormat::Json)]
    pub format: QuickstartOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct ChaosReportArgs {
    #[arg(short, long, default_value = "docs/correctness-report.html")]
    pub output: PathBuf,
    #[arg(long, default_value = "local")]
    pub source_revision: String,
    #[arg(long, default_value = "local")]
    pub source_repository: String,
    #[arg(long, default_value = "local")]
    pub workflow_run_url: String,
}
