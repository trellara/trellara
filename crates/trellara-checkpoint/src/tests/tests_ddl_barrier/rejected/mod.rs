pub(super) use super::*;

mod duplicate_required_ack;
mod lake_proof;
mod spark_proof;
mod stale_or_mismatched;
mod target_proof;

fn duplicate_ack(
    ack_lsn: &str,
    schema_version: &str,
    accepted: bool,
    detail: &str,
) -> DdlBarrierAck {
    DdlBarrierAck {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        barrier_id: "ddl-barrier-duplicate-required".to_string(),
        sink: "target_postgres".to_string(),
        ack_lsn: ack_lsn.to_string(),
        schema_version: schema_version.to_string(),
        accepted,
        detail: detail.to_string(),
    }
}
