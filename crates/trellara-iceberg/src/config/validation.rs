use crate::{IcebergIntegrationError, Result};

pub(super) fn validate_identifier_component(
    component: &str,
    field: &'static str,
    identifier: &str,
) -> Result<()> {
    if component.is_empty() {
        return Err(IcebergIntegrationError::InvalidTableIdentifier {
            identifier: identifier.to_string(),
            reason: format!("{field} cannot be empty"),
        });
    }
    if component.trim() != component {
        return Err(IcebergIntegrationError::InvalidTableIdentifier {
            identifier: identifier.to_string(),
            reason: format!("{field} cannot contain leading or trailing whitespace"),
        });
    }
    if component.chars().any(char::is_control) {
        return Err(IcebergIntegrationError::InvalidTableIdentifier {
            identifier: identifier.to_string(),
            reason: format!("{field} cannot contain control characters"),
        });
    }
    Ok(())
}

pub(super) fn validate_non_blank(
    field: &'static str,
    value: &str,
    error: fn(&'static str, String) -> IcebergIntegrationError,
) -> Result<()> {
    if value.trim().is_empty() {
        return Err(error(field, "cannot be blank".to_string()));
    }
    if value.trim() != value {
        return Err(error(
            field,
            "cannot contain leading or trailing whitespace".to_string(),
        ));
    }
    if value.chars().any(char::is_control) {
        return Err(error(
            field,
            "cannot contain control characters".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn validate_optional_non_blank(
    field: &'static str,
    value: Option<&str>,
    error: fn(&'static str, String) -> IcebergIntegrationError,
) -> Result<()> {
    if let Some(value) = value {
        validate_non_blank(field, value, error)?;
    }
    Ok(())
}

pub(super) fn validate_http_uri(
    field: &'static str,
    uri: &str,
    error: fn(&'static str, String) -> IcebergIntegrationError,
) -> Result<()> {
    validate_non_blank(field, uri, error)?;
    let Some((scheme, rest)) = uri.split_once("://") else {
        return Err(error(field, "must be an absolute URI".to_string()));
    };
    if !matches!(scheme, "http" | "https") {
        return Err(error(field, "must use http or https".to_string()));
    }
    if rest.is_empty() {
        return Err(error(field, "must include a host".to_string()));
    }
    Ok(())
}

pub(super) fn catalog_config_error(field: &'static str, reason: String) -> IcebergIntegrationError {
    IcebergIntegrationError::InvalidCatalogConfig { field, reason }
}

pub(super) fn object_store_config_error(
    field: &'static str,
    reason: String,
) -> IcebergIntegrationError {
    IcebergIntegrationError::InvalidObjectStoreConfig { field, reason }
}
