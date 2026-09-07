use std::path::PathBuf;

use clap::Parser;

use crate::QuickstartOutputFormat;

#[derive(Clone, Debug, Parser)]
pub struct FleetReportArgs {
    #[arg(short, long, required = true)]
    pub config: Vec<PathBuf>,
    #[arg(long, value_enum, default_value_t = QuickstartOutputFormat::Json)]
    pub format: QuickstartOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct FleetScorecardArgs {
    #[arg(short, long, required = true)]
    pub config: Vec<PathBuf>,
    #[arg(long, value_enum, default_value_t = QuickstartOutputFormat::Json)]
    pub format: QuickstartOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct FleetControlPlaneArgs {
    #[arg(short, long, required = true)]
    pub config: Vec<PathBuf>,
    #[arg(long, value_enum, default_value_t = QuickstartOutputFormat::Json)]
    pub format: QuickstartOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct FleetIdentityAuditArgs {
    #[arg(short, long, required = true)]
    pub config: Vec<PathBuf>,
    #[arg(long, value_enum, default_value_t = QuickstartOutputFormat::Json)]
    pub format: QuickstartOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct FleetEvidencePlanArgs {
    #[arg(short, long, required = true)]
    pub config: Vec<PathBuf>,
    #[arg(long, value_enum, default_value_t = QuickstartOutputFormat::Json)]
    pub format: QuickstartOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct FleetInitArgs {
    #[arg(long)]
    pub source_database_url: String,
    #[arg(long)]
    pub target_database_url: Option<String>,
    #[arg(long, default_value = "design-partner-fleet")]
    pub fleet_id: String,
    #[arg(long, default_value = "fleet-source")]
    pub source_id: String,
    #[arg(long, default_value = "postgres")]
    pub database_id: String,
    #[arg(long, required = true)]
    pub dataset: Vec<String>,
    #[arg(long, required = true)]
    pub table: Vec<String>,
    #[arg(long, default_value = "id")]
    pub primary_key: String,
    #[arg(long, default_value = "target/trellara-fleet")]
    pub output_dir: PathBuf,
    #[arg(long)]
    pub force: bool,
}
