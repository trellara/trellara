use crate::TrellaraConfig;

pub(crate) const SAMPLE_ACK_LSN: &str = "0/16B9000";
pub(crate) const SAMPLE_LAKE_EPOCH_ID: &str = "epoch-schema-ddl-sample";
pub(crate) const SAMPLE_LAKE_MANIFEST_DIGEST: &str =
    "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
pub(crate) const SAMPLE_SPARK_TEMPLATE_DIGEST: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
pub(crate) const SAMPLE_SPARK_ACCEPTED_BY: &str = "pilot-package-generator";
pub(crate) const SAMPLE_SPARK_VIEW_COUNT: u32 = 4;

pub(crate) fn raw_cdc_metadata_table(config: &TrellaraConfig) -> String {
    let dataset = config.dataset.id.replace('-', "_");
    format!("{dataset}__trellara__fanin___trellara_epoch_sources")
}

pub(crate) fn raw_cdc_partition_metadata_table(config: &TrellaraConfig) -> String {
    let dataset = config.dataset.id.replace('-', "_");
    format!("{dataset}__trellara__fanin___trellara_epoch_partitions")
}
