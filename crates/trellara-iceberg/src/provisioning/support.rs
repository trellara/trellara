use std::collections::HashMap;

use apache_iceberg::{Catalog, NamespaceIdent, TableIdent};

use super::DesiredTable;
use crate::{
    IcebergIntegrationError, IcebergTableIdentifier, IcebergTableProvisioningOutcome,
    IcebergTableProvisioningStatus, Result,
};

pub(super) async fn ensure_namespace(
    catalog: &dyn Catalog,
    target: &IcebergTableIdentifier,
) -> Result<()> {
    for depth in 1..=target.namespace.len() {
        let namespace = NamespaceIdent::from_vec(target.namespace[..depth].to_vec())
            .map_err(|error| catalog_error("build_namespace", target, error))?;
        if catalog
            .namespace_exists(&namespace)
            .await
            .map_err(|error| catalog_error("namespace_exists", target, error))?
        {
            continue;
        }
        if let Err(create_error) = catalog.create_namespace(&namespace, HashMap::new()).await {
            let exists = catalog
                .namespace_exists(&namespace)
                .await
                .map_err(|reload_error| IcebergIntegrationError::Catalog {
                    operation: "reconcile_create_namespace",
                    target: target.qualified_name(),
                    message: format!(
                        "create returned {create_error}; existence check failed: {reload_error}"
                    ),
                })?;
            if !exists {
                return Err(catalog_error("create_namespace", target, create_error));
            }
        }
    }
    Ok(())
}

pub(super) fn table_ident(target: &IcebergTableIdentifier) -> Result<TableIdent> {
    let mut components = target.namespace.clone();
    components.push(target.name.clone());
    TableIdent::from_strs(components)
        .map_err(|error| catalog_error("build_table_identifier", target, error))
}

pub(super) fn namespace_ident(target: &IcebergTableIdentifier) -> Result<NamespaceIdent> {
    NamespaceIdent::from_vec(target.namespace.clone())
        .map_err(|error| catalog_error("build_namespace_identifier", target, error))
}

pub(super) fn outcome(
    desired: &DesiredTable<'_>,
    status: IcebergTableProvisioningStatus,
) -> IcebergTableProvisioningOutcome {
    IcebergTableProvisioningOutcome {
        target: desired.target.clone(),
        schema_fingerprint_sha256: desired.schema_fingerprint.to_string(),
        status,
    }
}

pub(super) fn incompatible<T>(desired: &DesiredTable<'_>, reason: String) -> Result<T> {
    Err(IcebergIntegrationError::IncompatibleTable {
        target: desired.target.qualified_name(),
        reason,
    })
}

pub(super) fn catalog_error(
    operation: &'static str,
    target: &IcebergTableIdentifier,
    error: impl std::fmt::Display,
) -> IcebergIntegrationError {
    IcebergIntegrationError::Catalog {
        operation,
        target: target.qualified_name(),
        message: error.to_string(),
    }
}
