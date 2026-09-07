use super::types::QualificationFailurePoint;

impl QualificationFailurePoint {
    pub const ALL: [Self; 5] = [
        Self::SourcePromotionWhileRelayDisconnected,
        Self::BrokerOutageQuorumLoss,
        Self::TargetRestartDuringApply,
        Self::ObjectStoreSuccessCatalogTimeout,
        Self::TwentyFourHourSoakLargeTransactionMemoryCeiling,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::SourcePromotionWhileRelayDisconnected => {
                "qualification_source_promotion_while_relay_disconnected"
            }
            Self::BrokerOutageQuorumLoss => "qualification_broker_outage_quorum_loss",
            Self::TargetRestartDuringApply => "qualification_target_restart_during_apply",
            Self::ObjectStoreSuccessCatalogTimeout => {
                "qualification_object_store_success_catalog_timeout"
            }
            Self::TwentyFourHourSoakLargeTransactionMemoryCeiling => {
                "qualification_twenty_four_hour_soak_large_transaction_memory_ceiling"
            }
        }
    }

    pub(crate) fn boundary(self) -> &'static str {
        match self {
            Self::SourcePromotionWhileRelayDisconnected => "source_failover_slot_to_relay_restart",
            Self::BrokerOutageQuorumLoss => "broker_quorum_publish_ack",
            Self::TargetRestartDuringApply => "target_apply_transaction_commit",
            Self::ObjectStoreSuccessCatalogTimeout => {
                "object_store_write_before_catalog_visibility"
            }
            Self::TwentyFourHourSoakLargeTransactionMemoryCeiling => {
                "large_transaction_stream_spill_memory_ceiling"
            }
        }
    }

    pub(crate) fn invariant(self) -> &'static str {
        match self {
            Self::SourcePromotionWhileRelayDisconnected => {
                "promoted_source_replays_from_last_durable_ack"
            }
            Self::BrokerOutageQuorumLoss => "source_ack_waits_for_broker_quorum",
            Self::TargetRestartDuringApply => "target_restart_replays_uncheckpointed_apply",
            Self::ObjectStoreSuccessCatalogTimeout => {
                "catalog_timeout_cannot_publish_uncommitted_epoch"
            }
            Self::TwentyFourHourSoakLargeTransactionMemoryCeiling => {
                "soak_large_transactions_stay_within_memory_ceiling"
            }
        }
    }

    pub(crate) fn recovery_command(self) -> &'static str {
        match self {
            Self::SourcePromotionWhileRelayDisconnected => {
                "trellara source-safety --config <flow> --format text; trellara relay --config <flow>"
            }
            Self::BrokerOutageQuorumLoss => {
                "trellara status --config <flow> --view alerts --format text; trellara relay --config <flow>"
            }
            Self::TargetRestartDuringApply => {
                "trellara status --config <flow> --view diagnostics --format text; trellara apply --config <flow>"
            }
            Self::ObjectStoreSuccessCatalogTimeout => {
                "trellara lake fanin verify --config <flow> --format json"
            }
            Self::TwentyFourHourSoakLargeTransactionMemoryCeiling => {
                "trellara performance --config <flow> --format json; trellara chaos report --output docs/correctness-report.html"
            }
        }
    }
}
