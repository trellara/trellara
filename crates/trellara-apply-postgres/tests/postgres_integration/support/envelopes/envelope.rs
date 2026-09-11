use super::*;

pub(crate) fn envelope(
    transaction_id: &str,
    begin_lsn: &str,
    commit_lsn: &str,
    changes: Vec<ChangeRecord>,
) -> TransactionEnvelope {
    let schema_versions = schema_versions_for_changes(&changes);
    let mut envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: SOURCE_ID.to_string(),
        database_id: DATABASE_ID.to_string(),
        dataset_id: DATASET_ID.to_string(),
        transaction_id: transaction_id.to_string(),
        begin_lsn: begin_lsn.to_string(),
        commit_lsn: commit_lsn.to_string(),
        commit_timestamp_ms: 1_786_497_600_000,
        changes,
    });
    envelope.schema_versions = schema_versions;
    envelope.finalize_checksum();
    envelope
}

fn schema_versions_for_changes(changes: &[ChangeRecord]) -> Vec<RelationSchemaVersion> {
    let mut schema_versions = Vec::<RelationSchemaVersion>::new();
    for relation in changes.iter().filter_map(|change| change.relation.as_ref()) {
        let already_recorded = schema_versions.iter().any(|schema_version| {
            schema_version
                .relation
                .as_ref()
                .is_some_and(|existing| existing.display_name() == relation.display_name())
        });
        if !already_recorded {
            schema_versions.push(RelationSchemaVersion {
                relation: Some(relation.clone()),
                version: 12_345,
            });
        }
    }
    schema_versions
}

pub(crate) fn with_partition_manifest(
    mut envelope: TransactionEnvelope,
    partitions: Vec<ManifestPartition>,
) -> TransactionEnvelope {
    let affected_tables = affected_tables_for_changes(&envelope.changes);
    envelope.manifest = Some(TransactionManifest {
        transaction_id: envelope.transaction_id.clone(),
        source_commit_lsn: envelope.commit_lsn.clone(),
        source_commit_timestamp_ms: envelope.commit_timestamp_ms,
        global_event_count: envelope.changes.len() as u32,
        partitions,
        affected_tables,
        boundary_mode: ManifestBoundaryMode::PartitionedScale as i32,
    });
    envelope.finalize_checksum();
    envelope
}

fn affected_tables_for_changes(changes: &[ChangeRecord]) -> Vec<AffectedTable> {
    let mut affected_tables = Vec::<AffectedTable>::new();
    for change in changes {
        let relation = change.relation.clone().expect("change relation");
        if let Some(table) = affected_tables
            .iter_mut()
            .find(|table| table.relation.as_ref() == Some(&relation))
        {
            table.event_count += 1;
        } else {
            affected_tables.push(AffectedTable {
                relation: Some(relation),
                event_count: 1,
            });
        }
    }
    affected_tables
}

pub(crate) fn manifest_partition(
    id: u32,
    first_total_order: u32,
    last_total_order: u32,
) -> ManifestPartition {
    ManifestPartition {
        id,
        event_count: last_total_order - first_total_order + 1,
        first_total_order,
        last_total_order,
        checksum: u64::from(id) + u64::from(first_total_order) + u64::from(last_total_order),
    }
}
