use std::fmt;
use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Clone, Debug, Parser)]
#[command(
    name = "trellara-check",
    about = "Run a read-only PostgreSQL CDC source-safety diagnostic"
)]
pub struct CheckArgs {
    pub database_url: String,
    #[arg(long, value_enum, default_value_t = CheckOutputFormat::Text)]
    pub format: CheckOutputFormat,
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum CheckOutputFormat {
    Html,
    Json,
    Text,
}

impl fmt::Display for CheckOutputFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Html => "html",
            Self::Json => "json",
            Self::Text => "text",
        };
        formatter.write_str(value)
    }
}
