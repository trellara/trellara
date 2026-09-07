pub(crate) mod capture;
pub(crate) mod direct;
pub(crate) mod html;
pub(crate) mod init;
pub(crate) mod labels;
pub(crate) mod postgres_risk_text;
pub(crate) mod postgres_risks;
pub(crate) mod render;
pub(crate) mod score;
pub(crate) mod slot;
pub(crate) mod status;
pub(crate) mod subscription;
pub(crate) mod tables;
pub(crate) mod text;
pub(crate) mod types;

use trellara_pg_capture::PgCapture;

use crate::{
    direct_source_safety_capture_config, maybe_write_source_safety_init_config,
    source_safety::types::*, source_safety_init_recommendation, Result, SourceSafetyArgs,
};

pub(crate) async fn source_safety_from_database(
    args: &SourceSafetyArgs,
) -> Result<DirectSourceSafetySummary> {
    let direct_config = direct_source_safety_capture_config(args)?;
    let capture_kind = direct_config.capture_kind;
    let capture_config = direct_config.capture_config;
    let capture = PgCapture::connect(capture_config).await?;
    let tables = capture.inspect_tables().await?;
    let init_recommendation = source_safety_init_recommendation(args, &tables);
    let slot = capture
        .inspect_slot_status(capture_kind.expected_plugin())
        .await?;
    let subscription_conflicts = capture.inspect_subscription_conflict_stats().await?;

    let mut summary = DirectSourceSafetySummary::from_source_inspection(
        args.source_id.clone(),
        args.dataset_id.clone(),
        args.wal_retention_warn_bytes,
        tables.clone(),
        slot,
        subscription_conflicts,
        init_recommendation,
    );
    maybe_write_source_safety_init_config(args, &tables, &mut summary)?;

    Ok(summary)
}
