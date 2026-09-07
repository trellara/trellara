use super::*;

#[test]
fn strict_chunked_plan_preserves_source_order_under_manifest() {
    let envelope = multi_store_envelope();
    let plan = plan_strict_chunked_transaction(
        &envelope,
        &StrictChunkPlanConfig {
            max_changes_per_chunk: 2,
        },
    )
    .expect("strict chunk plan");

    assert_eq!(plan.manifest.transaction_id, "tx-1");
    assert_eq!(plan.manifest.source_commit_lsn, "0/16B6C50");
    assert_eq!(plan.manifest.global_event_count, 4);
    assert_eq!(
        ManifestBoundaryMode::try_from(plan.manifest.boundary_mode)
            .unwrap_or(ManifestBoundaryMode::Unspecified),
        ManifestBoundaryMode::StrictChunkedTransactionOrder
    );
    assert_eq!(plan.manifest.affected_tables.len(), 1);
    assert_eq!(plan.manifest.affected_tables[0].event_count, 4);
    assert_eq!(plan.chunks.len(), 2);
    assert_eq!(plan.manifest.partitions.len(), 2);
    assert_eq!(plan.manifest.partitions[0].id, 0);
    assert_eq!(plan.manifest.partitions[0].first_total_order, 1);
    assert_eq!(plan.manifest.partitions[0].last_total_order, 2);
    assert_eq!(plan.manifest.partitions[1].id, 1);
    assert_eq!(plan.manifest.partitions[1].first_total_order, 3);
    assert_eq!(plan.manifest.partitions[1].last_total_order, 4);

    let reconstructed =
        reconstruct_barrier_transaction(&plan.manifest, &plan.chunks).expect("reconstruct");
    assert_eq!(
        reconstructed
            .iter()
            .map(|change| change.total_order)
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 4]
    );
}

#[test]
fn strict_chunk_plan_rejects_empty_transaction_boundary() {
    let error = plan_strict_chunked_transaction(
        &empty_envelope(),
        &StrictChunkPlanConfig {
            max_changes_per_chunk: 2,
        },
    )
    .expect_err("empty strict chunk transaction");

    assert!(matches!(
        error,
        ProtocolError::EmptyTransaction { transaction_id } if transaction_id == "tx-empty"
    ));
}

#[test]
fn strict_chunked_plan_rejects_zero_chunk_size() {
    let envelope = multi_store_envelope();

    assert!(matches!(
        plan_strict_chunked_transaction(
            &envelope,
            &StrictChunkPlanConfig {
                max_changes_per_chunk: 0,
            },
        ),
        Err(ProtocolError::InvalidStrictChunkSize)
    ));
}
