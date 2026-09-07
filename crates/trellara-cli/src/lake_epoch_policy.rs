use crate::LakeEpochScenario;

pub(crate) fn lake_epoch_straggler_policy_decision(
    policy: &str,
    scenario: LakeEpochScenario,
) -> String {
    match (policy, scenario) {
        ("wait_all_required", LakeEpochScenario::LateStoreRecoveryCompletesEpoch) => {
            "late sources arrived before final publication; epoch is recomputed complete before Spark consumption"
                .to_string()
        }
        ("publish_with_gaps", LakeEpochScenario::OfflineStoresPublishWithGaps) => {
            "offline required sources remain explicit gaps and Spark jobs require accept_complete_with_gaps"
                .to_string()
        }
        ("publish_with_gaps", LakeEpochScenario::DuplicateStoreTransactionReplay) => {
            "duplicate replay is deduplicated by source transaction boundary before epoch publication"
                .to_string()
        }
        ("publish_with_gaps", LakeEpochScenario::ConflictingDuplicateQuarantine) => {
            "conflicting duplicate evidence overrides gap publication and quarantines the epoch"
                .to_string()
        }
        ("quarantine_on_gap", _) => {
            "any missing required source blocks publication until quarantine is resolved".to_string()
        }
        _ => "epoch publication follows the configured lake straggler policy".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_decision_names_gap_acceptance_for_offline_sources() {
        let decision = lake_epoch_straggler_policy_decision(
            "publish_with_gaps",
            LakeEpochScenario::OfflineStoresPublishWithGaps,
        );

        assert!(decision.contains("explicit gaps"));
        assert!(decision.contains("accept_complete_with_gaps"));
    }

    #[test]
    fn policy_decision_names_recomputed_complete_for_late_sources() {
        let decision = lake_epoch_straggler_policy_decision(
            "wait_all_required",
            LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
        );

        assert!(decision.contains("recomputed complete"));
    }
}
