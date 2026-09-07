use std::path::PathBuf;

use clap::Parser;

#[derive(Clone, Debug, Parser)]
pub struct QuarantineListArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long, default_value_t = 20)]
    pub limit: i64,
}

#[derive(Clone, Debug, Parser)]
pub struct QuarantineClearArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long)]
    pub transaction_id: String,
    #[arg(long)]
    pub commit_lsn: String,
}

#[derive(Clone, Debug, Parser)]
pub struct QuarantineReplayReadyArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long)]
    pub transaction_id: String,
    #[arg(long)]
    pub commit_lsn: String,
}
