use trellara_sim::FleetFanInFailurePoint;

use crate::LakeEpochScenario;

impl LakeEpochScenario {
    pub(crate) fn failure_point(self) -> FleetFanInFailurePoint {
        match self {
            Self::OfflineStoresPublishWithGaps => {
                FleetFanInFailurePoint::OfflineStoresPublishWithGaps
            }
            Self::LateStoreRecoveryCompletesEpoch => {
                FleetFanInFailurePoint::LateStoreRecoveryCompletesEpoch
            }
            Self::DuplicateStoreTransactionReplay => {
                FleetFanInFailurePoint::DuplicateStoreTransactionReplay
            }
            Self::ConflictingDuplicateQuarantine => {
                FleetFanInFailurePoint::ConflictingDuplicateQuarantine
            }
        }
    }

    pub(crate) fn test_name(self) -> &'static str {
        match self {
            Self::OfflineStoresPublishWithGaps => {
                "fleet_fanin_offline_stores_publish_with_explicit_gap_state"
            }
            Self::LateStoreRecoveryCompletesEpoch => {
                "fleet_fanin_late_sources_recompute_epoch_to_complete"
            }
            Self::DuplicateStoreTransactionReplay => {
                "fleet_fanin_duplicate_store_replay_is_deduplicated"
            }
            Self::ConflictingDuplicateQuarantine => {
                "fleet_fanin_conflicting_duplicate_quarantines_epoch"
            }
        }
    }
}
