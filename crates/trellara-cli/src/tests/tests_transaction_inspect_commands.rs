use super::*;

#[tokio::test]
async fn inspect_transaction_command_reads_encoded_envelope() {
    let file = std::env::temp_dir().join(format!(
        "trellara-inspect-{}-{}.pb",
        std::process::id(),
        "tx-inspect"
    ));
    let envelope = inspect_envelope();
    let manifest_checksum = envelope
        .manifest
        .as_ref()
        .expect("manifest")
        .compute_checksum();
    fs::write(&file, envelope.encode_checked().expect("encoded envelope")).expect("write envelope");
    let cli = Cli::try_parse_from([
        "trellara",
        "inspect-transaction",
        "--file",
        file.to_str().expect("utf8 path"),
    ])
    .expect("cli parse");

    let output = execute(cli).await.expect("execute");

    assert!(output.contains("\"transaction_id\": \"tx-inspect\""));
    assert!(output.contains("\"transaction_boundary\""));
    assert!(output.contains("\"status\": \"verified\""));
    assert!(output.contains("\"global_event_count_matches\": true"));
    assert!(output.contains("\"relation\": \"public.orders\""));
    assert!(output.contains("\"participating_partition_count\": 2"));
    assert!(output.contains(&format!(
        "\"expected_commit_marker_manifest_checksum\": {manifest_checksum}"
    )));
    assert!(output.contains(&format!("\"checksum\": {manifest_checksum}")));
    fs::remove_file(file).expect("remove envelope");
}

#[tokio::test]
async fn inspect_transaction_command_renders_text_boundary_evidence() {
    let file = std::env::temp_dir().join(format!(
        "trellara-inspect-text-{}-{}.pb",
        std::process::id(),
        "tx-inspect"
    ));
    let envelope = inspect_envelope();
    fs::write(&file, envelope.encode_checked().expect("encoded envelope")).expect("write envelope");
    let cli = Cli::try_parse_from([
        "trellara",
        "inspect-transaction",
        "--file",
        file.to_str().expect("utf8 path"),
        "--format",
        "text",
    ])
    .expect("cli parse");

    let output = execute(cli).await.expect("execute");

    assert!(output.contains("transaction: tx-inspect status=verified"));
    assert!(output.contains("mode=partitioned_scale_mode"));
    assert!(output.contains("events: dml=3 ddl=0 source_total=3"));
    assert!(output.contains("manifest_barrier_required=true"));
    assert!(output.contains("partition_event_count_matches=true"));
    assert!(output.contains("affected_tables:"));
    fs::remove_file(file).expect("remove envelope");
}
