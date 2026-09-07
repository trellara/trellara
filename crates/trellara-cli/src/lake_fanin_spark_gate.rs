use crate::{LakeEpochSummary, LakeFaninVerifyStatus};

use super::{
    lake_fanin_consumer_gate::lake_epoch_consumer_decision,
    lake_fanin_recovery_gate::non_consumable_epoch_gate,
};

pub(crate) fn spark_consumption_allowed(
    status: LakeFaninVerifyStatus,
    lake_epoch: &LakeEpochSummary,
    accept_complete_with_gaps: bool,
) -> bool {
    status == LakeFaninVerifyStatus::Match
        && lake_epoch_consumer_decision(lake_epoch, accept_complete_with_gaps).is_ok()
}

pub(crate) fn spark_consumption_gate(
    status: LakeFaninVerifyStatus,
    lake_epoch: &LakeEpochSummary,
    accept_complete_with_gaps: bool,
) -> String {
    if status != LakeFaninVerifyStatus::Match {
        return "blocked: stream and lake epoch proof artifacts do not match".to_string();
    }
    match lake_epoch_consumer_decision(lake_epoch, accept_complete_with_gaps) {
        Ok(decision) if decision.accepted_gaps => {
            return "released: stream and lake proofs match, complete_with_gaps was explicitly accepted, and verification_status=match"
                .to_string();
        }
        Ok(_) => {}
        Err(trellara_lake::LakeError::EpochNotConsumable { .. }) => {
            return non_consumable_epoch_gate(lake_epoch.state);
        }
        Err(trellara_lake::LakeError::EpochRequiresGapAcceptance { .. }) => {
            return "blocked: complete_with_gaps requires --accept-complete-with-gaps before Spark consumption"
                .to_string();
        }
        Err(trellara_lake::LakeError::EpochVerificationNotMatched { .. }) => {}
        Err(error) => return format!("blocked: {error}"),
    }
    if lake_epoch.verification_status != trellara_lake::LakeEpochVerificationStatus::Match {
        return format!(
            "blocked: lake verification_status {} is not match",
            crate::lake_epoch_verification_status_label(lake_epoch.verification_status)
        );
    }
    "released: stream and lake proofs match, epoch state is consumable, and verification_status=match"
        .to_string()
}
