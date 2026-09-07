use trellara_pg_capture::PgCapture;

use crate::{
    apply_preflight_contracts, bootstrap_configured_capture, fail_on_preflight_summary,
    BootstrapSummary, PreflightSummary, Result, SourceCaptureKind, TrellaraConfig,
};

pub(crate) async fn bootstrap_configured_snapshot_capture(
    config: &TrellaraConfig,
    create_if_missing: bool,
) -> Result<(
    BootstrapSummary,
    Option<trellara_pg_capture::ExportedLogicalSlot>,
)> {
    let capture_config = config.to_capture_config(create_if_missing)?;
    match config.source.capture {
        SourceCaptureKind::PgOutput if create_if_missing => {
            let bootstrap =
                PgCapture::bootstrap_with_exported_snapshot(capture_config.clone()).await?;
            let preflight = apply_preflight_contracts(config, bootstrap.preflight).await?;
            fail_on_preflight_summary(&preflight)?;
            let exported_snapshot_name = Some(bootstrap.exported_slot.snapshot_name.clone());
            let summary = BootstrapSummary {
                publication: bootstrap.publication_name,
                slot: bootstrap.exported_slot.slot_name.clone(),
                consistent_lsn: Some(bootstrap.exported_slot.consistent_lsn.clone()),
                exported_snapshot_name,
                relation_count: bootstrap.relations.len(),
                relations: bootstrap
                    .relations
                    .into_iter()
                    .map(|relation| relation.id.display_name())
                    .collect(),
                preflight: PreflightSummary::from_tables(preflight),
            };
            Ok((summary, Some(bootstrap.exported_slot)))
        }
        _ => Ok((
            bootstrap_configured_capture(config, create_if_missing).await?,
            None,
        )),
    }
}
