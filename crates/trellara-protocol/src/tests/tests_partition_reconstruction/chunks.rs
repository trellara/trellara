use super::*;

#[test]
fn barrier_reconstruction_rejects_missing_chunks() {
    let mut plan = partition_plan(8);
    let removed = plan.chunks.pop().expect("at least one chunk");

    assert!(matches!(
        reconstruct_barrier_transaction(&plan.manifest, &plan.chunks),
        Err(ProtocolError::MissingPartitionChunk { partition_id, .. })
            if partition_id == removed.partition_id
    ));
}

#[test]
fn barrier_reconstruction_rejects_chunks_not_listed_in_manifest() {
    let mut plan = partition_plan(8);
    let unlisted_partition_id = plan
        .manifest
        .partitions
        .iter()
        .map(|partition| partition.id)
        .max()
        .expect("partition")
        + 1;
    plan.chunks.push(PartitionChunk::new(
        "tx-1",
        unlisted_partition_id,
        vec![sale_change(99, "store-extra")],
    ));

    assert!(matches!(
        reconstruct_barrier_transaction(&plan.manifest, &plan.chunks),
        Err(ProtocolError::PartitionNotInManifest { partition_id, .. })
            if partition_id == unlisted_partition_id
    ));
}

#[test]
fn barrier_reconstruction_rejects_exact_duplicate_chunks() {
    let mut plan = partition_plan(8);
    let duplicate_partition_id = plan.chunks[0].partition_id;
    plan.chunks.push(plan.chunks[0].clone());

    assert!(matches!(
        reconstruct_barrier_transaction(&plan.manifest, &plan.chunks),
        Err(ProtocolError::DuplicatePartitionChunk { partition_id, .. })
            if partition_id == duplicate_partition_id
    ));
}

#[test]
fn barrier_reconstruction_rejects_conflicting_duplicate_chunks() {
    let mut plan = partition_plan(8);
    let mut duplicate = plan.chunks[0].clone();
    duplicate.changes.push(sale_change(99, "store-conflict"));
    duplicate.finalize_checksum();
    let duplicate_partition_id = duplicate.partition_id;
    plan.chunks.push(duplicate);

    assert!(matches!(
        reconstruct_barrier_transaction(&plan.manifest, &plan.chunks),
        Err(ProtocolError::DuplicatePartitionChunk { partition_id, .. })
            if partition_id == duplicate_partition_id
    ));
}

#[test]
fn barrier_reconstruction_rejects_chunk_checksum_tampering() {
    let mut plan = partition_plan(8);
    let partition_id = plan.chunks[0].partition_id;
    plan.chunks[0].checksum = plan.chunks[0].checksum.wrapping_add(1);

    assert!(matches!(
        reconstruct_barrier_transaction(&plan.manifest, &plan.chunks),
        Err(ProtocolError::PartitionChecksumMismatch { partition_id: actual, .. })
            if actual == partition_id
    ));
}

#[test]
fn barrier_reconstruction_rejects_padded_chunk_transaction_id_before_checksum_acceptance() {
    let mut plan = partition_plan(8);
    let partition_id = plan.chunks[0].partition_id;
    plan.chunks[0].transaction_id = " tx-1".to_string();
    plan.chunks[0].finalize_checksum();

    assert!(matches!(
        reconstruct_barrier_transaction(&plan.manifest, &plan.chunks),
        Err(ProtocolError::InvalidPartitionChunkField {
            partition_id: actual,
            field: "transaction_id",
            ..
        }) if actual == partition_id
    ));
}

#[test]
fn barrier_reconstruction_rejects_chunk_change_transaction_mismatch_after_checksum_acceptance() {
    let mut plan = partition_plan(8);
    let total_order = plan.chunks[0].changes[0].total_order;
    let partition_id = plan.chunks[0].partition_id;
    plan.chunks[0].changes[0].transaction_id = "tx-other".to_string();
    plan.chunks[0].finalize_checksum();
    plan.manifest
        .partitions
        .iter_mut()
        .find(|partition| partition.id == partition_id)
        .expect("manifest partition")
        .checksum = plan.chunks[0].checksum;

    assert!(matches!(
        reconstruct_barrier_transaction(&plan.manifest, &plan.chunks),
        Err(ProtocolError::ChangeTransactionMismatch {
            total_order: actual,
            expected,
            actual: found,
        }) if actual == total_order && expected == "tx-1" && found == "tx-other"
    ));
}
