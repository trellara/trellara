pub(crate) fn propagation_boundary_token(cdc_transaction_boundary: &str) -> Option<String> {
    field_token(cdc_transaction_boundary, "propagation_boundary")
}

pub(crate) fn propagation_decision_tokens(cdc_transaction_boundary: &str) -> Vec<String> {
    field_token(cdc_transaction_boundary, "propagation_decisions")
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn propagation_policy_sha256_token(cdc_transaction_boundary: &str) -> Option<String> {
    field_token(cdc_transaction_boundary, "propagation_policy_sha256")
        .filter(|value| value.len() == 64)
        .filter(|value| value.chars().all(|character| character.is_ascii_hexdigit()))
}

fn field_token(cdc_transaction_boundary: &str, field: &str) -> Option<String> {
    let prefix = format!("{field}=");
    cdc_transaction_boundary.split(';').find_map(|part| {
        let trimmed = part.trim();
        trimmed
            .strip_prefix(&prefix)
            .filter(|value| !value.trim().is_empty())
            .map(ToString::to_string)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_digest_token_requires_sha256_hex() {
        assert!(propagation_policy_sha256_token(&format!(
            "source commit LSN is the DDL barrier; propagation_policy_sha256={}",
            "a".repeat(64)
        ))
        .is_some());
        assert!(propagation_policy_sha256_token(
            "source commit LSN is the DDL barrier; propagation_policy_sha256=short"
        )
        .is_none());
    }
}
