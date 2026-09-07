use crate::{DdlBarrierReleaseAction, DdlBarrierReleaseBlocker};

pub(crate) fn ddl_barrier_release_actions(
    barrier_id: &str,
    barrier_lsn: &str,
    schema_version: &str,
    blockers: &[DdlBarrierReleaseBlocker],
) -> Vec<DdlBarrierReleaseAction> {
    blockers
        .iter()
        .filter_map(|blocker| release_action(barrier_id, barrier_lsn, schema_version, blocker))
        .collect()
}

fn release_action(
    barrier_id: &str,
    barrier_lsn: &str,
    schema_version: &str,
    blocker: &DdlBarrierReleaseBlocker,
) -> Option<DdlBarrierReleaseAction> {
    match blocker.code.as_str() {
        "pending_required_ack" => Some(DdlBarrierReleaseAction {
            code: "record_required_sink_ack".to_string(),
            sinks: blocker.sinks.clone(),
            command: format!(
                "trellara schema ddl-barrier ack --config <flow> --barrier-id {barrier_id} --sink <sink> --ack-lsn {barrier_lsn} --schema-version {schema_version}"
            ),
            reason: format!(
                "post-DDL DML remains invisible until required sinks {} acknowledge barrier_lsn {barrier_lsn}",
                csv(&blocker.sinks)
            ),
        }),
        "partition_visibility_not_released" => Some(DdlBarrierReleaseAction {
            code: "record_partition_visibility_ack".to_string(),
            sinks: blocker.sinks.clone(),
            command: format!(
                "trellara partition-watermarks --config <flow> && trellara schema ddl-barrier ack --config <flow> --barrier-id {barrier_id} --sink partition_visibility --ack-lsn {barrier_lsn} --schema-version {schema_version}"
            ),
            reason: "partitioned scale mode holds global post-DDL visibility until every partition lane reaches the schema barrier"
                .to_string(),
        }),
        "rejected_or_stale_ack" => Some(DdlBarrierReleaseAction {
            code: "replace_rejected_sink_ack".to_string(),
            sinks: blocker.sinks.clone(),
            command: format!(
                "trellara schema ddl-barrier ack --config <flow> --barrier-id {barrier_id} --sink <sink> --ack-lsn {barrier_lsn} --schema-version {schema_version}"
            ),
            reason: format!(
                "replace stale or mismatched ACK evidence for {} before post-DDL DML can release",
                csv(&blocker.sinks)
            ),
        }),
        "unexpected_ack" | "unexpected_sink_ack" => Some(DdlBarrierReleaseAction {
            code: "remove_unexpected_sink_ack".to_string(),
            sinks: blocker.sinks.clone(),
            command: "remove non-required ACK records or update the barrier required_sinks contract"
                .to_string(),
            reason: format!(
                "unexpected sink ACKs {} are outside the barrier release contract",
                csv(&blocker.sinks)
            ),
        }),
        "missing_required_sinks" => Some(DdlBarrierReleaseAction {
            code: "define_required_sinks_contract".to_string(),
            sinks: Vec::new(),
            command: "trellara schema ddl-barrier record --config <flow> --required-sink <sink>"
                .to_string(),
            reason: "post-DDL DML cannot release until the barrier declares its required sink ACK contract"
                .to_string(),
        }),
        _ => None,
    }
}

fn csv(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}
