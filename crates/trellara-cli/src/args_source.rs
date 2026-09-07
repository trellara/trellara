use std::path::PathBuf;

use clap::Parser;

use crate::SourceSafetyOutputFormat;

#[derive(Clone, Debug, Parser)]
pub struct SourceSafetyArgs {
    #[arg(short, long)]
    pub config: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = SourceSafetyOutputFormat::Json)]
    pub format: SourceSafetyOutputFormat,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long)]
    pub database_url: Option<String>,
    #[arg(long)]
    pub target_database_url: Option<String>,
    #[arg(long, default_value = "ad-hoc-source")]
    pub source_id: String,
    #[arg(long, default_value = "postgres")]
    pub database_id: String,
    #[arg(long, default_value = "ad-hoc")]
    pub dataset_id: String,
    #[arg(long, default_value = "trellara_publication")]
    pub publication: String,
    #[arg(long, default_value = "trellara_slot")]
    pub slot: String,
    #[arg(long, default_value = "pgoutput")]
    pub capture: String,
    #[arg(long)]
    pub wal_retention_warn_bytes: Option<i64>,
    #[arg(long, required_unless_present = "config")]
    pub table: Vec<String>,
    #[arg(long)]
    pub write_init: Option<PathBuf>,
    #[arg(long)]
    pub force: bool,
}

#[derive(Clone, Debug, Parser)]
pub struct InitArgs {
    #[arg(long)]
    pub source_database_url: String,
    #[arg(long)]
    pub target_database_url: Option<String>,
    #[arg(long, default_value = "local-source")]
    pub source_id: String,
    #[arg(long, default_value = "postgres")]
    pub database_id: String,
    #[arg(long, default_value = "default")]
    pub dataset_id: String,
    #[arg(long, default_value = "trellara_publication")]
    pub publication: String,
    #[arg(long, default_value = "trellara_slot")]
    pub slot: String,
    #[arg(long, default_value = "./target/trellara-local-stream")]
    pub stream_path: PathBuf,
    #[arg(long, default_value = "trellara.yml")]
    pub output: PathBuf,
    #[arg(long, default_value = "id")]
    pub primary_key: String,
    #[arg(long, required = true)]
    pub table: Vec<String>,
    #[arg(long)]
    pub evaluate: bool,
    #[arg(long)]
    pub force: bool,
}
