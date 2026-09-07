use trellara_lake::{
    build_epoch_summary, LakeEpoch, LakeEpochConfig, LakeError, LakeStragglerPolicy,
};
use trellara_protocol::TransactionEnvelope;

pub(crate) fn build_fleet_epoch_config(
    seed: u64,
    dataset_id: &str,
    sources: impl IntoIterator<Item = String>,
    straggler_policy: LakeStragglerPolicy,
) -> LakeEpochConfig {
    LakeEpochConfig::new(
        format!("epoch-{seed:016x}"),
        dataset_id.to_string(),
        sources,
        straggler_policy,
    )
}

pub(crate) fn summarize_fleet_epoch(
    config: &LakeEpochConfig,
    envelopes: &[TransactionEnvelope],
) -> Result<LakeEpoch, LakeError> {
    build_epoch_summary(config, envelopes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fleet_epoch_config_uses_stable_seeded_epoch_id_and_sources() {
        let config = build_fleet_epoch_config(
            42,
            "retail_sales",
            ["store-0001".to_string(), "store-0002".to_string()],
            LakeStragglerPolicy::WaitAllRequired,
        );

        assert_eq!(config.epoch_id, "epoch-000000000000002a");
        assert_eq!(config.dataset_id, "retail_sales");
        assert_eq!(config.required_sources.len(), 2);
        assert_eq!(
            config.straggler_policy,
            LakeStragglerPolicy::WaitAllRequired
        );
    }
}
