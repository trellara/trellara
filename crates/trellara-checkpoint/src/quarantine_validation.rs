use crate::checkpoint_validation::validate_flow_key;
use crate::transaction_key_validation::validate_transaction_key;
use crate::validation_guards::{require_non_empty, require_positive};
use crate::{ApplyQuarantine, FlowKey, Result, TransactionKey};

pub(crate) fn validate_quarantine_flow(flow: &FlowKey) -> Result<()> {
    validate_flow_key(flow)
}

pub(crate) fn validate_quarantine_limit(limit: i64) -> Result<()> {
    require_positive(
        limit,
        format!("quarantine list limit must be positive, got {limit}"),
    )
}

pub(crate) fn validate_quarantine_transaction(transaction: &TransactionKey) -> Result<()> {
    validate_transaction_key(transaction)
}

pub(crate) fn validate_quarantine_record(record: &ApplyQuarantine) -> Result<()> {
    validate_quarantine_transaction(&TransactionKey {
        source_id: record.source_id.clone(),
        database_id: record.database_id.clone(),
        dataset_id: record.dataset_id.clone(),
        transaction_id: record.transaction_id.clone(),
        commit_lsn: record.commit_lsn.clone(),
    })?;
    require_non_empty(&record.reason, "quarantine reason must not be empty")?;
    require_non_empty(&record.detail, "quarantine detail must not be empty")?;
    require_positive(
        record.attempt_count,
        format!(
            "quarantine attempt count must be positive, got {}",
            record.attempt_count
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transaction(commit_lsn: &str) -> TransactionKey {
        TransactionKey {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            transaction_id: "tx-1".to_string(),
            commit_lsn: commit_lsn.to_string(),
        }
    }

    #[test]
    fn quarantine_flow_rejects_empty_identity() {
        let error = validate_quarantine_flow(&FlowKey::new(" ", "sales"))
            .expect_err("empty source rejected");
        assert!(error
            .to_string()
            .contains("flow key source_id must not be empty"));
    }

    #[test]
    fn quarantine_flow_rejects_identity_with_surrounding_whitespace() {
        let error = validate_quarantine_flow(&FlowKey::new(" source-a ", "sales"))
            .expect_err("spaced source rejected");
        assert!(error
            .to_string()
            .contains("flow key source_id must not contain surrounding whitespace"));

        let error = validate_quarantine_flow(&FlowKey::new("source-a", " sales "))
            .expect_err("spaced dataset rejected");
        assert!(error
            .to_string()
            .contains("flow key dataset_id must not contain surrounding whitespace"));
    }

    #[test]
    fn quarantine_limit_rejects_non_positive_values() {
        let error = validate_quarantine_limit(0).expect_err("zero limit rejected");
        assert!(error.to_string().contains("limit must be positive"));
    }

    #[test]
    fn quarantine_transaction_rejects_invalid_boundary() {
        let error = validate_quarantine_transaction(&transaction("0/0"))
            .expect_err("zero commit lsn rejected");
        assert!(error.to_string().contains("commit_lsn"));
        assert!(error.to_string().contains("LSN must be greater than zero"));
    }

    #[test]
    fn quarantine_record_rejects_non_positive_attempt_count() {
        let mut record = valid_quarantine_record();
        record.attempt_count = 0;

        let error = validate_quarantine_record(&record).expect_err("invalid attempt count");

        assert!(error.to_string().contains("attempt count must be positive"));
    }

    fn valid_quarantine_record() -> ApplyQuarantine {
        ApplyQuarantine {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            transaction_id: "tx-1".to_string(),
            commit_lsn: "0/16B9000".to_string(),
            reason: "target_postgres_error".to_string(),
            detail: "duplicate key".to_string(),
            attempt_count: 1,
            last_seen_at: "2026-08-26T00:00:00Z".to_string(),
        }
    }
}
