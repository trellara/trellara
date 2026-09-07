use trellara_pg_capture::ReplicationSlotStatus;

use crate::{slot_evidence::slot_position_evidence, CheckFactor};

pub(crate) fn slot_failover_factors(slot: &ReplicationSlotStatus) -> Vec<CheckFactor> {
    let mut factors = Vec::new();
    if slot.failover == Some(false) {
        factors.push(CheckFactor::warning(
            "source_slot_failover_disabled",
            15,
            format!(
                "source replication slot {} is not configured as a failover slot; {}",
                slot.slot_name,
                slot_position_evidence(slot)
            ),
            "enable a failover logical slot or document the relay restart and reseed plan before relying on source promotion",
        ));
    }
    if slot.failover == Some(true) && slot.synced == Some(false) {
        factors.push(CheckFactor::warning(
            "source_slot_failover_not_synced",
            15,
            format!(
                "source replication slot {} is marked for failover but is not synced to the standby; {}",
                slot.slot_name,
                slot_position_evidence(slot)
            ),
            "wait for the failover slot to sync or repair standby slot synchronization before promoting a source",
        ));
    }
    factors
}
