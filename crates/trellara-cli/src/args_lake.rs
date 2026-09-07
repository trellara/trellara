use std::path::PathBuf;

use clap::Parser;

use crate::{LakeEpochScenario, QuickstartOutputFormat, DETERMINISTIC_EPOCH_ID};

#[derive(Clone, Debug, Parser)]
pub struct LakeInspectArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(short, long)]
    pub file: PathBuf,
}

#[derive(Clone, Debug, Parser)]
pub struct LakeWriterPlanArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long = "file", required = true)]
    pub files: Vec<PathBuf>,
    #[arg(long, default_value = DETERMINISTIC_EPOCH_ID)]
    pub epoch_id: String,
    #[arg(long, default_value_t = 32)]
    pub source_bucket_count: u32,
    #[arg(long, value_enum, default_value_t = QuickstartOutputFormat::Json)]
    pub format: QuickstartOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct LakeFaninRunArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long = "file", required = true)]
    pub files: Vec<PathBuf>,
    #[arg(long, default_value = DETERMINISTIC_EPOCH_ID)]
    pub epoch_id: String,
    #[arg(long, default_value_t = 32)]
    pub source_bucket_count: u32,
    #[arg(long, value_enum, default_value_t = QuickstartOutputFormat::Json)]
    pub format: QuickstartOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct LakeEpochArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long, value_enum, default_value_t = LakeEpochScenario::OfflineStoresPublishWithGaps)]
    pub scenario: LakeEpochScenario,
    #[arg(long, default_value_t = 12)]
    pub required_source_count: usize,
    #[arg(long, default_value_t = 3)]
    pub offline_source_count: usize,
    #[arg(long, default_value_t = 2)]
    pub duplicate_replay_count: usize,
    #[arg(long, value_enum, default_value_t = QuickstartOutputFormat::Json)]
    pub format: QuickstartOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct LakeFaninVerifyArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long)]
    pub stream_epoch: PathBuf,
    #[arg(long)]
    pub lake_epoch: PathBuf,
    #[arg(long)]
    pub accept_complete_with_gaps: bool,
    #[arg(long, value_enum, default_value_t = QuickstartOutputFormat::Json)]
    pub format: QuickstartOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct LakeFaninCompletenessArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long)]
    pub stream_epoch: PathBuf,
    #[arg(long)]
    pub lake_epoch: PathBuf,
    #[arg(long)]
    pub accept_complete_with_gaps: bool,
    #[arg(long, value_enum, default_value_t = QuickstartOutputFormat::Json)]
    pub format: QuickstartOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct LakeSparkTemplateArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long, default_value = "spark_catalog")]
    pub catalog: String,
    #[arg(long)]
    pub namespace: Option<String>,
    #[arg(long, help = "Configured source relation, for example public.sales")]
    pub table: Option<String>,
    #[arg(long, default_value = "<epoch_id>")]
    pub epoch_id: String,
    #[arg(long)]
    pub target_table: Option<String>,
    #[arg(long)]
    pub primary_key_column: Option<String>,
    #[arg(long)]
    pub accept_complete_with_gaps: bool,
    #[arg(long)]
    pub unsafe_allow_non_consumable_epoch: bool,
    #[arg(long)]
    pub unsafe_override_reason: Option<String>,
    #[arg(long, value_enum, default_value_t = QuickstartOutputFormat::Text)]
    pub format: QuickstartOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct LocalStreamSeekArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long)]
    pub topic: String,
    #[arg(long)]
    pub next_offset: i64,
    #[arg(long)]
    pub transaction_id: Option<String>,
    #[arg(long)]
    pub commit_lsn: Option<String>,
    #[arg(long)]
    pub consumer_group: Option<String>,
    #[arg(long)]
    pub allow_ahead: bool,
}

#[derive(Clone, Debug, Parser)]
pub struct LocalStreamLocateArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long)]
    pub transaction_id: String,
    #[arg(long)]
    pub commit_lsn: Option<String>,
    #[arg(long)]
    pub topic: Option<String>,
}

#[derive(Clone, Debug, Parser)]
pub struct LocalStreamReconstructArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long)]
    pub transaction_id: String,
    #[arg(long)]
    pub commit_lsn: Option<String>,
}
