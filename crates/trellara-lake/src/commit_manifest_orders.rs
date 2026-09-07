use std::collections::BTreeSet;

use trellara_protocol::TransactionEnvelope;

use crate::LakeError;

pub(crate) fn validate_manifest_order_boundaries(
    envelope: &TransactionEnvelope,
) -> Result<(), LakeError> {
    let Some(manifest) = &envelope.manifest else {
        return Ok(());
    };
    let orders = envelope
        .changes
        .iter()
        .map(|change| change.total_order)
        .collect::<BTreeSet<_>>();

    for partition in &manifest.partitions {
        if partition.event_count == 0 {
            continue;
        }
        require_order_boundary(
            &orders,
            partition.id,
            "first_total_order",
            partition.first_total_order,
        )?;
        require_order_boundary(
            &orders,
            partition.id,
            "last_total_order",
            partition.last_total_order,
        )?;
    }

    Ok(())
}

fn require_order_boundary(
    orders: &BTreeSet<u32>,
    partition_id: u32,
    field: &'static str,
    total_order: u32,
) -> Result<(), LakeError> {
    if orders.contains(&total_order) {
        Ok(())
    } else {
        Err(LakeError::ManifestOrderBoundaryMissing {
            partition_id,
            field,
            total_order,
        })
    }
}
