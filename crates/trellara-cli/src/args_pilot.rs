use std::path::PathBuf;

use clap::Parser;

use crate::PilotGuideOutputFormat;

#[derive(Clone, Debug, Parser)]
pub struct EvaluateArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long, value_enum, default_value_t = PilotGuideOutputFormat::Json)]
    pub format: PilotGuideOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct ConsistencyContractArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long, value_enum, default_value_t = PilotGuideOutputFormat::Json)]
    pub format: PilotGuideOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct PerformanceEnvelopeArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long, value_enum, default_value_t = PilotGuideOutputFormat::Json)]
    pub format: PilotGuideOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct IdentityAuditArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long, value_enum, default_value_t = PilotGuideOutputFormat::Json)]
    pub format: PilotGuideOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct PilotGuideArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long, value_enum, default_value_t = PilotGuideOutputFormat::Json)]
    pub format: PilotGuideOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct PilotScorecardArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long, value_enum, default_value_t = PilotGuideOutputFormat::Json)]
    pub format: PilotGuideOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct PilotEvidenceArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long, value_enum, default_value_t = PilotGuideOutputFormat::Json)]
    pub format: PilotGuideOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct PilotEvidenceCheckArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long, default_value = "target/trellara-live-evidence")]
    pub evidence_dir: PathBuf,
    #[arg(long, value_enum, default_value_t = PilotGuideOutputFormat::Json)]
    pub format: PilotGuideOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct PilotEvidenceTemplateArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(short, long, default_value = "target/trellara-live-evidence")]
    pub output: PathBuf,
    #[arg(long, value_enum, default_value_t = PilotGuideOutputFormat::Json)]
    pub format: PilotGuideOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct PilotPackageArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(short, long, default_value = "target/trellara-pilot-package")]
    pub output: PathBuf,
    #[arg(long, default_value = "docs/correctness-report.html")]
    pub correctness_report: PathBuf,
}

#[derive(Clone, Debug, Parser)]
pub struct EvidenceRegistryArgs {
    #[arg(short = 'p', long, default_value = "target/trellara-pilot-package")]
    pub package: PathBuf,
    #[arg(long, default_value = "docs/correctness-report.html")]
    pub correctness_report: PathBuf,
    #[arg(long, value_enum, default_value_t = PilotGuideOutputFormat::Json)]
    pub format: PilotGuideOutputFormat,
}
