use super::*;

#[test]
fn bounded_large_transaction_proof_accepts_partitioned_manifest_barrier() {
    let config = TrellaraConfig::from_yaml(&partitioned_yaml(), "test").expect("parse config");

    let gate = bounded_large_transaction_run_proof(Path::new("trellara.yml"), &config);

    assert_eq!(gate.status, RunProofStatus::Verified);
    assert_eq!(gate.code, "bounded_large_transaction_capture");
    assert!(gate.evidence.contains("mode=partitioned_scale_mode"));
    assert!(gate.evidence.contains("capture=pgoutput"));
    assert!(gate
        .evidence
        .contains("partition_manifest_barrier.partition_count=16"));
    assert!(gate.evidence.contains("key_column=store_id"));
    assert!(gate
        .evidence
        .contains("bounded_memory_contract=spill_then_partition_manifest"));
    assert!(gate.evidence.contains(
        "visibility_contract=global_visibility_waits_for_manifest_commit_marker_and_all_partitions"
    ));
}
