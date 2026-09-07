use trellara_lake::LakeStragglerPolicy;

const PUBLISH_WITH_GAPS_GRACE_MS: u64 = 300_000;

pub(crate) fn publish_with_gaps_policy() -> LakeStragglerPolicy {
    LakeStragglerPolicy::PublishWithGaps {
        grace_ms: PUBLISH_WITH_GAPS_GRACE_MS,
    }
}
