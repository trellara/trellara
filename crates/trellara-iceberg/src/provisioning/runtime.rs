use apache_iceberg::transaction::{AddColumn, ApplyTransactionAction, Transaction};
use apache_iceberg::Catalog;

use super::compatibility::{inspect_table, require_fully_compatible, Compatibility};
use super::schema::{iceberg_type, require_ddl_ack, table_creation};
use super::support::{
    catalog_error, ensure_namespace, incompatible, namespace_ident, outcome, table_ident,
};
use super::{
    DesiredTable, DDL_ACK_LSN_PROPERTY, DDL_BARRIER_PROPERTY, DDL_SCHEMA_VERSION_PROPERTY,
    SCHEMA_FINGERPRINT_PROPERTY,
};
use crate::{
    IcebergDdlAcknowledgement, IcebergIntegrationError, IcebergTableProvisioningOutcome,
    IcebergTableProvisioningStatus, Result,
};

pub(super) async fn provision_tables(
    catalog: &dyn Catalog,
    dataset_id: &str,
    tables: &[DesiredTable<'_>],
    ddl_ack: Option<&IcebergDdlAcknowledgement>,
) -> Result<Vec<IcebergTableProvisioningOutcome>> {
    let mut outcomes = Vec::with_capacity(tables.len());
    for table in tables {
        ensure_namespace(catalog, table.target).await?;
        outcomes.push(provision_table(catalog, dataset_id, table, ddl_ack).await?);
    }
    Ok(outcomes)
}

async fn provision_table(
    catalog: &dyn Catalog,
    dataset_id: &str,
    desired: &DesiredTable<'_>,
    ddl_ack: Option<&IcebergDdlAcknowledgement>,
) -> Result<IcebergTableProvisioningOutcome> {
    let ident = table_ident(desired.target)?;
    if !catalog
        .table_exists(&ident)
        .await
        .map_err(|error| catalog_error("table_exists", desired.target, error))?
    {
        let ack = require_ddl_ack(dataset_id, desired, ddl_ack, "create table")?;
        let creation = table_creation(desired, ack)?;
        return match catalog
            .create_table(&namespace_ident(desired.target)?, creation)
            .await
        {
            Ok(created) => {
                require_fully_compatible(&created, desired)?;
                Ok(outcome(desired, IcebergTableProvisioningStatus::Created))
            }
            Err(create_error) => {
                let reloaded = catalog.load_table(&ident).await.map_err(|reload_error| {
                    IcebergIntegrationError::Catalog {
                        operation: "reconcile_create_table",
                        target: desired.target.qualified_name(),
                        message: format!(
                            "create returned {create_error}; fresh load failed: {reload_error}"
                        ),
                    }
                })?;
                require_fully_compatible(&reloaded, desired)?;
                Ok(outcome(
                    desired,
                    IcebergTableProvisioningStatus::ReconciledAfterAmbiguousFailure,
                ))
            }
        };
    }
    let table = catalog
        .load_table(&ident)
        .await
        .map_err(|error| catalog_error("load_table", desired.target, error))?;
    let compatibility = inspect_table(&table, desired)?;
    if compatibility.missing_columns.is_empty() && compatibility.fingerprint_matches {
        return Ok(outcome(desired, IcebergTableProvisioningStatus::Compatible));
    }
    let ack = require_ddl_ack(dataset_id, desired, ddl_ack, "evolve schema")?;
    evolve_table(catalog, desired, &table, &ident, compatibility, ack).await
}

async fn evolve_table(
    catalog: &dyn Catalog,
    desired: &DesiredTable<'_>,
    table: &apache_iceberg::table::Table,
    ident: &apache_iceberg::TableIdent,
    compatibility: Compatibility<'_>,
    ack: &IcebergDdlAcknowledgement,
) -> Result<IcebergTableProvisioningOutcome> {
    let mut transaction = Transaction::new(table);
    if !compatibility.missing_columns.is_empty() {
        let first_expected_id = table
            .metadata()
            .last_column_id()
            .checked_add(1)
            .ok_or_else(|| IcebergIntegrationError::IncompatibleTable {
                target: desired.target.qualified_name(),
                reason: "catalog field IDs are exhausted".to_string(),
            })?;
        let mut update = transaction.update_schema();
        for (expected_id, column) in (first_expected_id..).zip(compatibility.missing_columns) {
            if column.required {
                return incompatible(
                    desired,
                    format!(
                        "required column {} cannot be added without a historical default",
                        column.name
                    ),
                );
            }
            if column.id != expected_id {
                return incompatible(desired, format!("additive evolution would assign field id {expected_id} to {}, but the writer contract requires {}", column.name, column.id));
            }
            update = update.add_column(AddColumn::optional(
                &column.name,
                iceberg_type(column.column_type),
            ));
        }
        transaction = update
            .apply(transaction)
            .map_err(|error| catalog_error("plan_schema_evolution", desired.target, error))?;
    }
    transaction = transaction
        .update_table_properties()
        .set(
            SCHEMA_FINGERPRINT_PROPERTY.to_string(),
            desired.schema_fingerprint.to_string(),
        )
        .set(DDL_BARRIER_PROPERTY.to_string(), ack.barrier_id.clone())
        .set(DDL_ACK_LSN_PROPERTY.to_string(), ack.ack_lsn.clone())
        .set(
            DDL_SCHEMA_VERSION_PROPERTY.to_string(),
            ack.schema_version.clone(),
        )
        .apply(transaction)
        .map_err(|error| catalog_error("plan_schema_properties", desired.target, error))?;
    let status = match transaction.commit(catalog).await {
        Ok(updated) => {
            require_fully_compatible(&updated, desired)?;
            IcebergTableProvisioningStatus::SchemaEvolved
        }
        Err(commit_error) => {
            let reloaded = catalog.load_table(ident).await.map_err(|reload_error| {
                IcebergIntegrationError::Catalog {
                    operation: "reconcile_schema_evolution",
                    target: desired.target.qualified_name(),
                    message: format!(
                        "schema commit returned {commit_error}; fresh load failed: {reload_error}"
                    ),
                }
            })?;
            require_fully_compatible(&reloaded, desired)?;
            IcebergTableProvisioningStatus::ReconciledAfterAmbiguousFailure
        }
    };
    Ok(outcome(desired, status))
}
