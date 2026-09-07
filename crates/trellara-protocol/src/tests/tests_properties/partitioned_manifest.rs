use super::*;

proptest! {
    #[test]
    fn partitioned_manifest_property_reconstructs_source_order(
        change_count in 1_u32..96,
        partition_count in 1_u32..16,
    ) {
        let envelope = generated_partitioned_envelope(change_count, partition_count);
        let plan = plan_partitioned_transaction(
            &envelope,
            &PartitionPlanConfig {
                partition_count,
                key_column: "store_id".to_string(),
                null_key_policy: PartitionNullKeyPolicy::Quarantine,
                key_change_policy: PartitionKeyChangePolicy::Quarantine,
            },
        ).expect("partitioned plan");

        prop_assert_eq!(
            ManifestBoundaryMode::try_from(plan.manifest.boundary_mode)
                .unwrap_or(ManifestBoundaryMode::Unspecified),
            ManifestBoundaryMode::PartitionedScale,
        );
        prop_assert_eq!(plan.manifest.global_event_count, change_count);
        prop_assert!(plan.chunks.len() <= partition_count as usize);
        for partition in &plan.manifest.partitions {
            let chunk = plan
                .chunks
                .iter()
                .find(|chunk| chunk.partition_id == partition.id)
                .expect("manifest chunk");
            prop_assert_eq!(partition.event_count, chunk.changes.len() as u32);
            prop_assert_eq!(partition.checksum, chunk.checksum);
            prop_assert!(chunk.verify_checksum().is_ok());
        }

        let reconstructed =
            reconstruct_barrier_transaction(&plan.manifest, &plan.chunks).expect("reconstruct");
        prop_assert_eq!(
            reconstructed.iter().map(|change| change.total_order).collect::<Vec<_>>(),
            (1..=change_count).collect::<Vec<_>>(),
        );
    }

    #[test]
    fn partitioned_manifest_property_rejects_missing_partition_chunk(
        change_count in 2_u32..96,
        partition_count in 2_u32..16,
    ) {
        let envelope = generated_partitioned_envelope(change_count, partition_count);
        let mut plan = plan_partitioned_transaction(
            &envelope,
            &PartitionPlanConfig {
                partition_count,
                key_column: "store_id".to_string(),
                null_key_policy: PartitionNullKeyPolicy::Quarantine,
                key_change_policy: PartitionKeyChangePolicy::Quarantine,
            },
        ).expect("partitioned plan");
        prop_assume!(plan.chunks.len() > 1);
        let removed = plan.chunks.pop().expect("removed chunk");

        let result = reconstruct_barrier_transaction(&plan.manifest, &plan.chunks);
        match result {
            Err(ProtocolError::MissingPartitionChunk { partition_id, .. }) => {
                prop_assert_eq!(partition_id, removed.partition_id);
            }
            other => prop_assert!(false, "expected missing partition chunk error, got {other:?}"),
        }
    }

    #[test]
    fn partitioned_manifest_property_rejects_duplicate_partition_chunk(
        change_count in 1_u32..96,
        partition_count in 1_u32..16,
    ) {
        let envelope = generated_partitioned_envelope(change_count, partition_count);
        let mut plan = plan_partitioned_transaction(
            &envelope,
            &PartitionPlanConfig {
                partition_count,
                key_column: "store_id".to_string(),
                null_key_policy: PartitionNullKeyPolicy::Quarantine,
                key_change_policy: PartitionKeyChangePolicy::Quarantine,
            },
        ).expect("partitioned plan");
        let duplicate_partition_id = plan.chunks[0].partition_id;
        plan.chunks.push(plan.chunks[0].clone());

        let result = reconstruct_barrier_transaction(&plan.manifest, &plan.chunks);
        match result {
            Err(ProtocolError::DuplicatePartitionChunk { partition_id, .. }) => {
                prop_assert_eq!(partition_id, duplicate_partition_id);
            }
            other => prop_assert!(false, "expected duplicate partition chunk error, got {other:?}"),
        }
    }
}
