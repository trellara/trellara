use serde_yaml::Value;

use crate::{migrate_config_yaml, CliError, Result};

pub fn redact_config_yaml(contents: &str, label: &str) -> Result<String> {
    let migrated = migrate_config_yaml(contents, label)?;
    let mut document: Value =
        serde_yaml::from_str(&migrated.yaml).map_err(|source| CliError::ParseConfig {
            path: label.to_string(),
            source,
        })?;
    redact_value(&mut document);
    serde_yaml::to_string(&document).map_err(|source| CliError::ParseConfig {
        path: label.to_string(),
        source,
    })
}

fn redact_value(value: &mut Value) {
    match value {
        Value::Mapping(mapping) => {
            for (name, child) in mapping {
                if name.as_str().is_some_and(is_sensitive_configuration_key) {
                    *child = Value::String(redaction_marker(child));
                } else {
                    redact_value(child);
                }
            }
        }
        Value::Sequence(values) => values.iter_mut().for_each(redact_value),
        _ => {}
    }
}

fn is_sensitive_configuration_key(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "database_url"
            | "password"
            | "username"
            | "ca_certificate"
            | "client_certificate"
            | "client_key"
            | "access_key"
            | "access_key_id"
            | "api_key"
            | "credential"
            | "secret_key"
            | "secret_access_key"
            | "session_token"
            | "token"
            | "credentials"
    )
}

fn redaction_marker(value: &Value) -> String {
    let source = value
        .as_mapping()
        .and_then(|mapping| mapping.get(Value::String("source".to_string())))
        .and_then(Value::as_str);
    match source {
        Some("environment_variable") => "<redacted:environment_variable>".to_string(),
        Some("file") => "<redacted:file>".to_string(),
        _ => "<redacted>".to_string(),
    }
}
