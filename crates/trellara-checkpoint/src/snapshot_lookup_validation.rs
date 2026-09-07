use crate::{
    checkpoint_validation::validate_flow_key,
    validation_guards::{require_no_surrounding_whitespace, require_non_empty},
    FlowKey, Result,
};

pub(crate) fn validate_snapshot_run_lookup(flow: &FlowKey, run_id: &str) -> Result<()> {
    validate_flow_key(flow)?;
    validate_lookup_field("snapshot run lookup", "run_id", run_id)
}

pub(crate) fn validate_latest_snapshot_run_lookup(flow: &FlowKey) -> Result<()> {
    validate_flow_key(flow)
}

pub(crate) fn validate_snapshot_table_progress_lookup(
    flow: &FlowKey,
    run_id: &str,
    relation: &str,
) -> Result<()> {
    validate_snapshot_table_progress_list_lookup(flow, run_id)?;
    validate_lookup_field("snapshot table progress lookup", "relation", relation)
}

pub(crate) fn validate_snapshot_table_progress_list_lookup(
    flow: &FlowKey,
    run_id: &str,
) -> Result<()> {
    validate_flow_key(flow)?;
    validate_lookup_field("snapshot table progress lookup", "run_id", run_id)
}

fn validate_lookup_field(context: &str, field: &'static str, value: &str) -> Result<()> {
    require_non_empty(value, format!("{context} {field} must not be empty"))?;
    require_no_surrounding_whitespace(
        value,
        format!("{context} {field} must not contain surrounding whitespace"),
    )
}
