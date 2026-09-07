use super::*;

pub(super) fn assert_qualification_boundaries(summary: &ChaosRunSummary) {
    let source_promotion = scenario(
        summary,
        "qualification_source_promotion_while_relay_disconnected",
    );
    assert_eq!(
        source_promotion.invariant,
        "promoted_source_replays_from_last_durable_ack"
    );
    assert_eq!(
        source_promotion.boundary_mode,
        "source_failover_slot_to_relay_restart"
    );
    assert!(source_promotion
        .expected_safety_property
        .contains("source acknowledgement lag remains visible"));

    let broker = scenario(summary, "qualification_broker_outage_quorum_loss");
    assert_eq!(broker.invariant, "source_ack_waits_for_broker_quorum");
    assert_eq!(broker.boundary_mode, "broker_quorum_publish_ack");
    assert!(broker.proof_command.contains("trellara-sim"));

    let target_restart = scenario(summary, "qualification_target_restart_during_apply");
    assert_eq!(
        target_restart.invariant,
        "target_restart_replays_uncheckpointed_apply"
    );
    assert_eq!(
        target_restart.boundary_mode,
        "target_apply_transaction_commit"
    );
    assert!(target_restart
        .recovery_command
        .as_deref()
        .is_some_and(|command| command.contains("trellara apply --config <flow>")));

    let catalog = scenario(
        summary,
        "qualification_object_store_success_catalog_timeout",
    );
    assert_eq!(
        catalog.invariant,
        "catalog_timeout_cannot_publish_uncommitted_epoch"
    );
    assert_eq!(
        catalog.boundary_mode,
        "object_store_write_before_catalog_visibility"
    );
    assert!(catalog
        .expected_safety_property
        .contains("pending_catalog_commit"));

    let soak = scenario(
        summary,
        "qualification_twenty_four_hour_soak_large_transaction_memory_ceiling",
    );
    assert_eq!(
        soak.invariant,
        "soak_large_transactions_stay_within_memory_ceiling"
    );
    assert_eq!(
        soak.boundary_mode,
        "large_transaction_stream_spill_memory_ceiling"
    );
    assert!(soak
        .expected_safety_property
        .contains("24-hour soak window"));
}
