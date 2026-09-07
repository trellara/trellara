use std::collections::HashMap;

use apache_iceberg::transaction::{ApplyTransactionAction, Transaction};
use apache_iceberg::Catalog;
use uuid::Uuid;

use crate::runtime_support::{
    data_file, matching_snapshot_id, receipt, table_ident, validate_snapshot_properties,
};
use crate::{
    IcebergIntegrationError, IcebergTableAppendPlan, IcebergTableCommitReceipt,
    IcebergTableCommitStatus, Result,
};

pub async fn commit_iceberg_table_append(
    catalog: &dyn Catalog,
    plan: &IcebergTableAppendPlan,
) -> Result<IcebergTableCommitReceipt> {
    let target = table_ident(plan)?;
    let table =
        catalog
            .load_table(&target)
            .await
            .map_err(|error| IcebergIntegrationError::Catalog {
                operation: "load_table",
                target: plan.target.qualified_name(),
                message: error.to_string(),
            })?;
    if let Some(snapshot_id) = matching_snapshot_id(&table, plan)? {
        return Ok(receipt(
            plan,
            snapshot_id,
            IcebergTableCommitStatus::AlreadyCommitted,
        ));
    }

    let files = plan
        .data_files
        .iter()
        .map(|file| data_file(&table, plan, file))
        .collect::<Result<Vec<_>>>()?;
    let commit_uuid = deterministic_commit_uuid(plan)?;
    let transaction = Transaction::new(&table);
    let action = transaction
        .fast_append()
        .with_check_duplicate(plan.requirements.check_duplicate_files)
        .set_commit_uuid(commit_uuid)
        .set_snapshot_properties(snapshot_properties(plan))
        .add_data_files(files);
    let transaction =
        action
            .apply(transaction)
            .map_err(|error| IcebergIntegrationError::Catalog {
                operation: "plan_fast_append",
                target: plan.target.qualified_name(),
                message: error.to_string(),
            })?;
    let committed = match transaction.commit(catalog).await {
        Ok(committed) => committed,
        Err(commit_error) => {
            return reconcile_ambiguous_commit(catalog, plan, commit_error.to_string()).await;
        }
    };
    let snapshot = committed.metadata().current_snapshot().ok_or_else(|| {
        IcebergIntegrationError::ConflictingCatalogEvidence {
            target: plan.target.qualified_name(),
            reason: "fast append returned a table without a current snapshot".to_string(),
        }
    })?;
    validate_snapshot_properties(plan, &snapshot.summary().additional_properties)?;
    Ok(receipt(
        plan,
        snapshot.snapshot_id(),
        IcebergTableCommitStatus::Committed,
    ))
}

async fn reconcile_ambiguous_commit(
    catalog: &dyn Catalog,
    plan: &IcebergTableAppendPlan,
    commit_error: String,
) -> Result<IcebergTableCommitReceipt> {
    let target = table_ident(plan)?;
    let reloaded = catalog.load_table(&target).await.map_err(|reload_error| {
        IcebergIntegrationError::Catalog {
            operation: "reconcile_ambiguous_commit",
            target: plan.target.qualified_name(),
            message: format!(
                "commit returned {commit_error}; fresh catalog read also failed: {reload_error}"
            ),
        }
    })?;
    if let Some(snapshot_id) = matching_snapshot_id(&reloaded, plan)? {
        return Ok(receipt(
            plan,
            snapshot_id,
            IcebergTableCommitStatus::AlreadyCommitted,
        ));
    }
    Err(IcebergIntegrationError::Catalog {
        operation: "commit_fast_append",
        target: plan.target.qualified_name(),
        message: format!(
            "{commit_error}; a fresh catalog read found no snapshot for table_commit_id {}",
            plan.table_commit_id
        ),
    })
}

fn deterministic_commit_uuid(plan: &IcebergTableAppendPlan) -> Result<Uuid> {
    Uuid::parse_str(&plan.commit_uuid).map_err(|error| {
        IcebergIntegrationError::ConflictingCatalogEvidence {
            target: plan.target.qualified_name(),
            reason: format!("invalid deterministic commit UUID: {error}"),
        }
    })
}

fn snapshot_properties(plan: &IcebergTableAppendPlan) -> HashMap<String, String> {
    plan.snapshot_properties
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}
