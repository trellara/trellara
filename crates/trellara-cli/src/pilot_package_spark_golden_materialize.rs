use crate::{SparkGoldenCurrentStateRow, SparkGoldenRawCdcRow, SparkGoldenScd2Row};

pub(crate) fn materialize_current_state(
    rows: &[SparkGoldenRawCdcRow],
) -> Vec<SparkGoldenCurrentStateRow> {
    let mut latest = Vec::<&SparkGoldenRawCdcRow>::new();
    for row in rows {
        let key = record_key(row);
        if let Some(existing) = latest
            .iter_mut()
            .find(|candidate| candidate.source_id == row.source_id && record_key(candidate) == key)
        {
            if row.total_order > existing.total_order {
                *existing = row;
            }
        } else {
            latest.push(row);
        }
    }

    latest
        .into_iter()
        .filter(|row| row.operation != "delete")
        .map(|row| SparkGoldenCurrentStateRow {
            source_id: row.source_id.clone(),
            relation: row.relation.clone(),
            record_key: record_key(row),
            transaction_id: row.transaction_id.clone(),
            commit_lsn: row.commit_lsn.clone(),
            idempotency_key: row.idempotency_key.clone(),
            row_after_json: row
                .payload_after
                .clone()
                .expect("non-delete current-state row has after image"),
        })
        .collect()
}

pub(crate) fn materialize_scd2(rows: &[SparkGoldenRawCdcRow]) -> Vec<SparkGoldenScd2Row> {
    let mut history = Vec::<SparkGoldenScd2Row>::new();
    for row in rows {
        if matches!(row.operation.as_str(), "update" | "delete") {
            close_current_version(&mut history, row);
        }
        if matches!(row.operation.as_str(), "insert" | "update") {
            history.push(SparkGoldenScd2Row {
                source_id: row.source_id.clone(),
                relation: row.relation.clone(),
                record_key: record_key(row),
                valid_from: row.commit_timestamp.clone(),
                valid_to: None,
                is_current: true,
                open_idempotency_key: row.idempotency_key.clone(),
                close_commit_lsn: None,
                row_after_json: row
                    .payload_after
                    .clone()
                    .expect("opening SCD2 row has after image"),
            });
        }
    }
    history
}

fn close_current_version(history: &mut [SparkGoldenScd2Row], row: &SparkGoldenRawCdcRow) {
    let key = record_key(row);
    if let Some(current) = history.iter_mut().rev().find(|candidate| {
        candidate.source_id == row.source_id && candidate.record_key == key && candidate.is_current
    }) {
        current.valid_to = Some(row.commit_timestamp.clone());
        current.is_current = false;
        current.close_commit_lsn = Some(row.commit_lsn.clone());
    }
}

fn record_key(row: &SparkGoldenRawCdcRow) -> String {
    let image = row
        .payload_after
        .as_deref()
        .or(row.payload_before.as_deref())
        .expect("golden row has before or after image");
    serde_json::from_str::<serde_json::Value>(image).expect("golden row image is valid json")["id"]
        .as_str()
        .expect("golden row id is string")
        .to_string()
}
