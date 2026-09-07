use trellara_pg_capture::ReplicationSlotStatus;

use crate::{source_safety::types::SourceSafetyFactor, source_slot_position_evidence};

pub(crate) fn source_slot_failover_factors(
    source_slot: &ReplicationSlotStatus,
) -> Vec<SourceSafetyFactor> {
    let mut factors = Vec::new();

    if source_slot.failover == Some(false) {
        factors.push(SourceSafetyFactor::warning(
            "source_slot_failover_disabled",
            15,
            format!(
                "source replication slot {} is not configured as a failover slot; {}",
                source_slot.slot_name,
                source_slot_position_evidence(source_slot)
            ),
            "enable a failover logical slot or document the relay restart and reseed plan before relying on source promotion",
        ));
    }

    if source_slot.failover == Some(true) && source_slot.synced == Some(false) {
        factors.push(SourceSafetyFactor::warning(
            "source_slot_failover_not_synced",
            15,
            format!(
                "source replication slot {} is marked for failover but is not synced to the standby; {}",
                source_slot.slot_name,
                source_slot_position_evidence(source_slot)
            ),
            "wait for the failover slot to sync or repair standby slot synchronization before promoting a source",
        ));
    }

    factors
}
