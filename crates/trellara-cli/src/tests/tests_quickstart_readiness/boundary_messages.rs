use super::*;

#[test]
fn quickstart_boundary_messages_cover_partitioned_mode_and_custom_spill_dir() {
    let yaml = local_partitioned_yaml()
        .replace("partition_count: 16", "partition_count: 8")
        .replace(
            "  database_url: postgresql://trellara:trellara@localhost:55432/trellara_source",
            "  database_url: postgresql://trellara:trellara@localhost:55432/trellara_source\n  stream_spill_threshold_changes: 2048\n  stream_spill_dir: /tmp/trellara-spill",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse config");

    assert_eq!(
        quickstart_transaction_boundary_message(&config),
        "partitioned manifest boundary enabled across 8 partitions"
    );
    assert_eq!(
        quickstart_capture_spill_message(&config),
        "pgoutput streamed transaction changes spill after 2048 changes into /tmp/trellara-spill"
    );
}
