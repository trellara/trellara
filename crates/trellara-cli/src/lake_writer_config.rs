use crate::TrellaraConfig;

pub(crate) fn raw_cdc_writer_config(
    config: &TrellaraConfig,
    epoch_id: impl Into<String>,
    source_bucket_count: u32,
) -> trellara_lake::LakeRawCdcWriterConfig {
    trellara_lake::LakeRawCdcWriterConfig::new(&config.dataset.id, epoch_id, source_bucket_count)
        .with_required_sources([config.source.id.clone()])
}
