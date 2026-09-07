use super::*;

#[test]
fn pilot_guide_packages_brokerless_design_partner_path() {
    let config = TrellaraConfig::from_yaml(&local_stream_yaml(), "test").expect("parse");
    let guide = PilotGuideSummary::from_config(&config, Path::new("trellara.yml"));

    assert_eq!(guide.source_id, "local-source");
    assert_eq!(guide.dataset_id, "retail-sales");
    assert_eq!(guide.stream_kind, "local");
    assert!(!guide.kafka_required);
    assert!(guide
        .capture_spill_boundary
        .contains("pgoutput streamed transaction changes spill after 1024 changes"));
    assert!(guide.capture_spill_boundary.contains("OS temp directory"));
    assert_eq!(
        guide.evaluation_time_budget_minutes,
        QUICKSTART_TIME_BUDGET_MINUTES
    );
    assert_eq!(guide.table_count, 1);
    assert!(guide.phases.iter().any(|phase| {
        phase.name == "brokerless verified loop"
            && phase
                .command
                .contains("trellara run --local --verify --format text --config trellara.yml")
    }));
    assert!(guide.evidence_commands.contains(
        &"trellara status --config trellara.yml --view diagnostics --format text".to_string()
    ));
    assert!(guide
        .large_transaction_evidence
        .contains(&"trellara quickstart --config trellara.yml --check".to_string()));
    assert!(guide
        .large_transaction_evidence
        .contains(&"trellara contract-test --config trellara.yml".to_string()));
    assert!(guide
        .large_transaction_evidence
        .contains(&"trellara chaos run".to_string()));
    assert!(guide.large_transaction_evidence.contains(
        &"trellara status --config trellara.yml --view report --format text".to_string()
    ));
    assert!(guide
        .large_transaction_evidence
        .contains(&"trellara stream inspect-local --config trellara.yml".to_string()));
    assert!(guide
        .failure_drill
        .contains(&"trellara stream inspect-local --config trellara.yml".to_string()));
    assert!(guide
        .acceptance_gates
        .iter()
        .any(|gate| gate.contains("10 minutes or less")));
    assert!(guide
        .acceptance_gates
        .iter()
        .any(|gate| gate.contains("without adopting Kafka")));
    assert!(guide
        .acceptance_gates
        .iter()
        .any(|gate| gate.contains("stream inspect-local reports clean or recovered")));
    assert!(guide
        .acceptance_gates
        .iter()
        .any(|gate| gate.contains("large transactions stay bounded")));
}

#[test]
fn pilot_guide_surfaces_strict_chunk_large_transaction_gate() {
    let yaml = local_stream_yaml().replace(
        "  tables:\n    - schema: public\n      name: sales",
        "  strict_chunking:\n    max_changes_per_chunk: 1000\n  tables:\n    - schema: public\n      name: sales",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let guide = PilotGuideSummary::from_config(&config, Path::new("trellara.yml"));

    assert!(guide
        .transaction_boundary
        .contains("one source transaction is chunked at 1000 changes per chunk"));
    assert!(guide.acceptance_gates.iter().any(|gate| gate
        .contains("strict chunk manifests make oversized transactions")
        && gate.contains("commit marker")));
}

#[test]
fn pilot_guide_includes_partition_watermark_gate() {
    let yaml = STRICT_YAML
        .replace("strict_transaction_order", "partitioned_scale_mode")
        .replace(
            "  tables:\n    - schema: public\n      name: sales",
            "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 4\n    key_column: store_id",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let guide =
        PilotGuideSummary::from_config(&config, Path::new("examples/retail-fleet/partitioned.yml"));

    assert!(guide.kafka_required);
    assert_eq!(guide.stream_kind, "kafka");
    assert!(guide.evidence_commands.contains(
        &"trellara partition-watermarks --config examples/retail-fleet/partitioned.yml".to_string()
    ));
    assert!(guide
        .acceptance_gates
        .iter()
        .any(|gate| gate.contains("every partition complete")));
}

#[test]
fn pilot_guide_text_renders_human_evaluation_package() {
    let config = TrellaraConfig::from_yaml(&local_stream_yaml(), "test").expect("parse");
    let guide = PilotGuideSummary::from_config(&config, Path::new("trellara.yml"));
    let output = render_pilot_guide_text(&guide);

    assert!(output.contains("Trellara pilot guide"));
    assert!(output.contains("source: local-source"));
    assert!(output.contains("kafka_required: false"));
    assert!(output.contains("capture_spill_boundary:"));
    assert!(output.contains("phases:"));
    assert!(output.contains("trellara check --config trellara.yml"));
    assert!(output.contains("trellara run --local --verify --format text --config trellara.yml"));
    assert!(output.contains("evidence_commands:"));
    assert!(output.contains("large_transaction_evidence:"));
    assert!(output.contains("trellara quickstart --config trellara.yml --check"));
    assert!(output.contains("failure_drill:"));
    assert!(output.contains("acceptance_gates:"));
    assert!(output.contains("pilot can run without adopting Kafka"));
}
