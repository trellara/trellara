use std::path::PathBuf;

use clap::Parser;

use crate::{DdlPlanApplyMode, QuickstartOutputFormat};

#[derive(Clone, Debug, Parser)]
pub struct DdlPlanArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(
        long = "change",
        required = true,
        help = "Proposed DDL change, for example add_nullable_column:public.sales.discount_code:text"
    )]
    pub changes: Vec<String>,
    #[arg(long, value_enum, default_value_t = DdlPlanApplyMode::ManualReview)]
    pub apply_mode: DdlPlanApplyMode,
    #[arg(long, value_enum, default_value_t = QuickstartOutputFormat::Json)]
    pub format: QuickstartOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct DdlEnvelopePlanArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long)]
    pub file: PathBuf,
    #[arg(long, value_enum, default_value_t = QuickstartOutputFormat::Json)]
    pub format: QuickstartOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct DdlBarrierRecordArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(
        long = "change",
        required = true,
        help = "Approved DDL change, for example add_nullable_column:public.sales.discount_code:text"
    )]
    pub changes: Vec<String>,
    #[arg(long, value_enum, default_value_t = DdlPlanApplyMode::AutoSafe)]
    pub apply_mode: DdlPlanApplyMode,
    #[arg(long)]
    pub barrier_lsn: String,
    #[arg(long)]
    pub schema_version: String,
    #[arg(long, value_enum, default_value_t = QuickstartOutputFormat::Json)]
    pub format: QuickstartOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct DdlBarrierAckArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long)]
    pub barrier_id: String,
    #[arg(long)]
    pub sink: String,
    #[arg(long)]
    pub ack_lsn: String,
    #[arg(long)]
    pub schema_version: String,
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub accepted: bool,
    #[arg(long, default_value = "accepted")]
    pub detail: String,
    #[arg(long)]
    pub plan_sha256: Option<String>,
    #[arg(
        long = "statement-sha256",
        help = "Target DDL statement SHA-256 digest from schema ddl-apply-plan; repeat once per statement"
    )]
    pub statement_sha256s: Vec<String>,
    #[arg(long)]
    pub epoch_id: Option<String>,
    #[arg(long)]
    pub metadata_table: Option<String>,
    #[arg(long)]
    pub partition_metadata_table: Option<String>,
    #[arg(long)]
    pub manifest_digest: Option<String>,
    #[arg(long)]
    pub template_digest: Option<String>,
    #[arg(long)]
    pub accepted_by: Option<String>,
    #[arg(long)]
    pub view_count: Option<u32>,
    #[arg(long)]
    pub barrier_lsn: Option<String>,
    #[arg(long)]
    pub expected_partition_count: Option<u32>,
    #[arg(
        long = "partition-durable-lsn",
        help = "Partition durability evidence as <partition_id>=<durable_lsn>; repeat once per partition"
    )]
    pub partition_durable_lsns: Vec<String>,
    #[arg(
        long = "partition-applied-lsn",
        help = "Partition watermark evidence as <partition_id>=<applied_lsn>; repeat once per partition"
    )]
    pub partition_applied_lsns: Vec<String>,
    #[arg(long, value_enum, default_value_t = QuickstartOutputFormat::Json)]
    pub format: QuickstartOutputFormat,
}

#[derive(Clone, Debug, Parser)]
pub struct DdlBarrierStatusArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long)]
    pub barrier_id: String,
    #[arg(long, value_enum, default_value_t = QuickstartOutputFormat::Json)]
    pub format: QuickstartOutputFormat,
}
