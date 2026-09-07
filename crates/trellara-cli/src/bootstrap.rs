use trellara_pg_capture::{CaptureBootstrap, PgCapture, TestDecodingCapture};

use crate::{
    apply_preflight_contracts, fail_on_preflight_summary, BootstrapSummary, PreflightSummary,
    Result, SourceCaptureKind, TrellaraConfig,
};

pub(crate) async fn bootstrap_configured_capture(
    config: &TrellaraConfig,
    create_if_missing: bool,
) -> Result<BootstrapSummary> {
    let capture_config = config.to_capture_config(create_if_missing)?;
    let bootstrap = match config.source.capture {
        SourceCaptureKind::PgOutput => PgCapture::bootstrap(capture_config).await?,
        SourceCaptureKind::TestDecoding => {
            TestDecodingCapture::connect(capture_config.clone()).await?;
            let capture = PgCapture::connect(capture_config.clone()).await?;
            let preflight = capture.ensure_capture_safe().await?;
            let relations = capture.load_relations().await?;
            trellara_pg_capture::CaptureBootstrap {
                publication_name: capture_config.publication_name,
                slot_name: capture_config.slot_name,
                consistent_lsn: None,
                exported_snapshot_name: None,
                relations,
                preflight,
            }
        }
    };
    capture_bootstrap_summary(config, bootstrap).await
}

pub(crate) async fn capture_bootstrap_summary(
    config: &TrellaraConfig,
    bootstrap: CaptureBootstrap,
) -> Result<BootstrapSummary> {
    let preflight = apply_preflight_contracts(config, bootstrap.preflight).await?;
    fail_on_preflight_summary(&preflight)?;
    Ok(BootstrapSummary {
        publication: bootstrap.publication_name,
        slot: bootstrap.slot_name,
        consistent_lsn: bootstrap.consistent_lsn,
        exported_snapshot_name: bootstrap.exported_snapshot_name,
        relation_count: bootstrap.relations.len(),
        relations: bootstrap
            .relations
            .into_iter()
            .map(|relation| relation.id.display_name())
            .collect(),
        preflight: PreflightSummary::from_tables(preflight),
    })
}
