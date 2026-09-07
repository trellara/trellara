use trellara_protocol::{ColumnValue, Operation, ReplicaIdentity, RowImage};

use crate::test_decoding_relation::{parse_relation_names, parse_single_relation_name};
use crate::{CaptureError, LogicalEvent, Result};

pub(crate) fn parse_test_decoding_event(lsn: &str, data: &str) -> Result<LogicalEvent> {
    if let Some(transaction_id) = data.strip_prefix("BEGIN ") {
        return Ok(LogicalEvent::Begin {
            transaction_id: transaction_id.to_string(),
            begin_lsn: lsn.to_string(),
        });
    }

    if data.strip_prefix("COMMIT ").is_some() {
        return Ok(LogicalEvent::Commit {
            commit_lsn: lsn.to_string(),
            commit_timestamp_ms: 0,
        });
    }

    parse_test_decoding_change(data)
}

fn parse_test_decoding_change(data: &str) -> Result<LogicalEvent> {
    let rest = data.strip_prefix("table ").ok_or_else(|| {
        CaptureError::TestDecodingParse(format!("expected table change, got {data:?}"))
    })?;
    let (relation_name, rest) = rest.split_once(": ").ok_or_else(|| {
        CaptureError::TestDecodingParse(format!("missing relation separator in {data:?}"))
    })?;
    let (operation, rest) = rest.split_once(": ").ok_or_else(|| {
        CaptureError::TestDecodingParse(format!("missing operation separator in {data:?}"))
    })?;

    match operation {
        "INSERT" => {
            let relation = parse_single_relation_name(relation_name)?;
            Ok(LogicalEvent::Change {
                transaction_id: None,
                relation,
                operation: Operation::Insert,
                replica_identity: ReplicaIdentity::Full,
                before: None,
                after: Some(RowImage::new(parse_test_decoding_columns(rest)?)),
            })
        }
        "UPDATE" => {
            let relation = parse_single_relation_name(relation_name)?;
            let (before, after) = rest.split_once(" new-tuple: ").ok_or_else(|| {
                CaptureError::TestDecodingParse(format!(
                    "missing update new-tuple marker in {data:?}"
                ))
            })?;
            let before = before.strip_prefix("old-key: ").ok_or_else(|| {
                CaptureError::TestDecodingParse(format!(
                    "missing update old-key marker in {data:?}"
                ))
            })?;
            Ok(LogicalEvent::Change {
                transaction_id: None,
                relation,
                operation: Operation::Update,
                replica_identity: ReplicaIdentity::Full,
                before: Some(RowImage::new(parse_test_decoding_columns(before)?)),
                after: Some(RowImage::new(parse_test_decoding_columns(after)?)),
            })
        }
        "DELETE" => {
            let relation = parse_single_relation_name(relation_name)?;
            Ok(LogicalEvent::Change {
                transaction_id: None,
                relation,
                operation: Operation::Delete,
                replica_identity: ReplicaIdentity::Full,
                before: Some(RowImage::new(parse_test_decoding_columns(rest)?)),
                after: None,
            })
        }
        "TRUNCATE" => Ok(LogicalEvent::Truncate {
            transaction_id: None,
            relations: parse_relation_names(relation_name)?,
        }),
        other => Err(CaptureError::TestDecodingParse(format!(
            "unsupported test_decoding operation {other:?}"
        ))),
    }
}

fn parse_test_decoding_columns(input: &str) -> Result<Vec<ColumnValue>> {
    let mut columns = Vec::new();
    let mut rest = input.trim();

    while !rest.is_empty() {
        let (name, after_name) = rest.split_once('[').ok_or_else(|| {
            CaptureError::TestDecodingParse(format!("missing column type start in {rest:?}"))
        })?;
        let (type_name, after_type) = after_name.split_once("]:").ok_or_else(|| {
            CaptureError::TestDecodingParse(format!("missing column type end in {rest:?}"))
        })?;
        let after_type = after_type.trim_start();
        let (value, after_value) = parse_test_decoding_value(after_type)?;
        columns.push(ColumnValue::text(
            name,
            type_oid_hint(type_name),
            value,
            is_probable_key_column(name),
        ));
        rest = after_value.trim_start();
    }

    Ok(columns)
}

fn parse_test_decoding_value(input: &str) -> Result<(String, &str)> {
    let input = input.strip_prefix('\'').ok_or_else(|| {
        CaptureError::TestDecodingParse(format!("expected quoted value in {input:?}"))
    })?;
    let end = input.find('\'').ok_or_else(|| {
        CaptureError::TestDecodingParse(format!("unterminated quoted value in {input:?}"))
    })?;
    Ok((input[..end].to_string(), &input[end + 1..]))
}

fn type_oid_hint(type_name: &str) -> u32 {
    match type_name {
        "text" => 25,
        "integer" => 23,
        "bigint" => 20,
        "timestamp with time zone" => 1184,
        _ => 0,
    }
}

fn is_probable_key_column(name: &str) -> bool {
    name == "id" || name.ends_with("_id")
}
