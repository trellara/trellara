use serde_json::Value;

use super::json_path;

pub(super) fn transaction_boundary_present(value: &Value) -> bool {
    boundary_at(value, &["cdc_transaction_boundary"]).is_some_and(holds_post_ddl_dml)
        || boundary_at(value, &["release_summary", "cdc_transaction_boundary"])
            .is_some_and(holds_post_ddl_dml)
}

pub(super) fn holds_post_ddl_dml(boundary: &str) -> bool {
    boundary.contains("source commit LSN is the DDL barrier")
        && boundary.contains("post-DDL DML stays invisible")
        && (boundary.contains("required sink ACK")
            || boundary.contains("required acknowledgements"))
        && boundary.contains("barrier_lsn")
}

fn boundary_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    json_path(Some(value), path).and_then(Value::as_str)
}
