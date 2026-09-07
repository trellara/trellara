use crate::pilot_package_spark_golden::SparkGoldenRawCdcRow;

pub(crate) fn raw_cdc_rows(epoch_id: &str) -> Vec<SparkGoldenRawCdcRow> {
    vec![
        raw_row(
            epoch_id,
            "store-001",
            "tx-001",
            "0/00000010",
            "2026-08-16T00:01:00Z",
            1,
            "insert",
            None,
            Some(r#"{"id":"sale-100","amount":"25.00","status":"open"}"#),
        ),
        raw_row(
            epoch_id,
            "store-001",
            "tx-002",
            "0/00000020",
            "2026-08-16T00:02:00Z",
            2,
            "update",
            Some(r#"{"id":"sale-100","amount":"25.00","status":"open"}"#),
            Some(r#"{"id":"sale-100","amount":"30.00","status":"paid"}"#),
        ),
        raw_row(
            epoch_id,
            "store-002",
            "tx-003",
            "0/00000030",
            "2026-08-16T00:03:00Z",
            3,
            "insert",
            None,
            Some(r#"{"id":"sale-200","amount":"14.00","status":"open"}"#),
        ),
        raw_row(
            epoch_id,
            "store-002",
            "tx-004",
            "0/00000040",
            "2026-08-16T00:04:00Z",
            4,
            "delete",
            Some(r#"{"id":"sale-200","amount":"14.00","status":"open"}"#),
            None,
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
fn raw_row(
    epoch_id: &str,
    source_id: &str,
    transaction_id: &str,
    commit_lsn: &str,
    commit_timestamp: &str,
    total_order: u64,
    operation: &str,
    payload_before: Option<&str>,
    payload_after: Option<&str>,
) -> SparkGoldenRawCdcRow {
    SparkGoldenRawCdcRow {
        epoch_id: epoch_id.to_string(),
        source_id: source_id.to_string(),
        relation: "public.sales".to_string(),
        transaction_id: transaction_id.to_string(),
        commit_lsn: commit_lsn.to_string(),
        commit_timestamp: commit_timestamp.to_string(),
        total_order,
        operation: operation.to_string(),
        idempotency_key: format!("{source_id}:{transaction_id}:{total_order}"),
        payload_before: payload_before.map(ToString::to_string),
        payload_after: payload_after.map(ToString::to_string),
    }
}
