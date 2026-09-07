use super::*;

pub(super) fn assert_source_failover_contract(summary: &ChaosRunSummary) {
    let failover = scenario(summary, "source_failover_after_publish_before_ack");
    assert_eq!(
        failover.invariant,
        "failover_slot_replay_preserves_transaction_boundary"
    );
    assert_eq!(
        failover.boundary_mode,
        "strict_transaction_order_with_failover_slot"
    );
    assert!(failover.proof_command.contains("trellara-sim"));

    let unsynced_failover_slot = scenario(summary, "source_failover_slot_unsynced");
    assert_eq!(
        unsynced_failover_slot.invariant,
        "failover_slot_sync_before_promotion"
    );
    assert_eq!(
        unsynced_failover_slot.boundary_mode,
        "source_failover_readiness"
    );
    assert!(unsynced_failover_slot
        .proof_command
        .contains("source_safety_warns_when_failover_slot_is_not_synced"));

    let disabled_failover_slot = scenario(summary, "source_failover_slot_disabled");
    assert_eq!(
        disabled_failover_slot.invariant,
        "failover_slot_enabled_before_promotion"
    );
    assert_eq!(
        disabled_failover_slot.boundary_mode,
        "source_failover_readiness"
    );
    assert!(disabled_failover_slot
        .proof_command
        .contains("source_safety_warns_when_failover_slot_is_disabled"));
    assert!(disabled_failover_slot
        .proof_command
        .contains("direct_source_safety_warns_when_failover_slot_is_disabled"));
    assert!(disabled_failover_slot
        .evidence
        .contains("source_safety_warns_when_failover_slot_is_disabled"));
    assert!(disabled_failover_slot
        .evidence
        .contains("direct_source_safety_warns_when_failover_slot_is_disabled"));

    let abandoned_slot = scenario(summary, "source_slot_abandoned_idle_timeout");
    assert_eq!(
        abandoned_slot.invariant,
        "inactive_slot_cleanup_posture_is_explicit"
    );
    assert_eq!(abandoned_slot.boundary_mode, "source_safety_slot_lifecycle");
    assert!(abandoned_slot
        .proof_command
        .contains("direct_source_safety_mentions_disabled_idle_slot_timeout"));
    assert!(abandoned_slot
        .proof_command
        .contains("direct_source_safety_surfaces_configured_idle_slot_timeout"));
    assert!(abandoned_slot
        .expected_safety_property
        .contains("idle_replication_slot_timeout"));
    assert_eq!(
        abandoned_slot.recovery_command.as_deref(),
        Some("trellara-check <source> --format text")
    );
}
