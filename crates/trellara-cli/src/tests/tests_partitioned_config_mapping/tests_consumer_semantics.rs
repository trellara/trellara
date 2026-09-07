use super::*;

#[test]
fn consumer_semantics_summary_marks_exact_transaction_native_for_strict_mode() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let summary =
        ConsumerSemanticsSummary::from_config(&config, Path::new("trellara.yml")).expect("summary");

    assert_eq!(summary.dataset_mode, "strict_transaction_order");
    assert_eq!(summary.selected_consumer_mode, "exact_transaction");
    assert_eq!(summary.partition_key, None);
    assert!(summary
        .strict_language
        .contains("preserves source transaction order"));
    assert_eq!(summary.matrix.len(), 5);
    let exact = summary
        .matrix
        .iter()
        .find(|mode| mode.mode == "exact_transaction")
        .expect("exact mode");
    assert_eq!(exact.availability, ConsumerSemanticsAvailability::Native);
    assert!(exact
        .transaction_boundary
        .contains("complete source transaction envelope"));
    assert!(exact.ordering.contains("global source transaction order"));
    let partition_local = summary
        .matrix
        .iter()
        .find(|mode| mode.mode == "partition_local")
        .expect("partition local mode");
    assert_eq!(
        partition_local.availability,
        ConsumerSemanticsAvailability::NotRecommended
    );
    assert!(partition_local
        .visibility_rule
        .contains("partition lanes are not configured"));
}

#[test]
fn consumer_semantics_summary_explains_partitioned_tradeoffs() {
    let yaml = STRICT_YAML
            .replace("strict_transaction_order", "partitioned_scale_mode")
            .replace(
                "  tables:\n    - schema: public\n      name: sales",
                "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 4\n    key_column: store_id\n    null_key_policy: derive_from_primary_key\n    key_change_policy: emit_move",
            );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let summary = ConsumerSemanticsSummary::from_config(&config, Path::new("partitioned.yml"))
        .expect("summary");

    assert_eq!(summary.dataset_mode, "partitioned_scale_mode");
    assert_eq!(summary.selected_consumer_mode, "barrier_aware");
    assert_eq!(summary.partition_key.as_deref(), Some("store_id"));
    assert_eq!(summary.partition_count, Some(4));
    assert_eq!(
        summary.null_key_policy.as_deref(),
        Some("derive_from_primary_key")
    );
    assert_eq!(summary.key_change_policy.as_deref(), Some("emit_move"));
    assert_eq!(
        summary.manifest_topic.as_deref(),
        Some("trellara.local-source.retail-sales.manifest")
    );
    assert_eq!(
        summary.commit_topic.as_deref(),
        Some("trellara.local-source.retail-sales.commit")
    );
    assert!(summary
        .partitioned_language
        .contains("consumers choose between barrier-aware atomic processing"));
    let exact = summary
        .matrix
        .iter()
        .find(|mode| mode.mode == "exact_transaction")
        .expect("exact mode");
    assert_eq!(
        exact.availability,
        ConsumerSemanticsAvailability::SupportedWithBarrier
    );
    assert!(exact
        .visibility_rule
        .contains("every participating partition chunk"));
    let partition_local = summary
        .matrix
        .iter()
        .find(|mode| mode.mode == "partition_local")
        .expect("partition local mode");
    assert_eq!(
        partition_local.availability,
        ConsumerSemanticsAvailability::Native
    );
    assert!(partition_local
        .not_guaranteed
        .contains(&"atomic visibility for cross-partition transactions".to_string()));
    let current_state = summary
        .matrix
        .iter()
        .find(|mode| mode.mode == "current_state")
        .expect("current-state mode");
    assert!(current_state
        .visibility_rule
        .contains("lowest complete partition watermark"));
    assert!(summary
        .recommended_next_steps
        .contains(&"trellara partition-watermarks --config partitioned.yml".to_string()));
}
