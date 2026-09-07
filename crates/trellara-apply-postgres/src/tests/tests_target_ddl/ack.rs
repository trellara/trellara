use super::*;
use trellara_checkpoint::{DdlBarrier, DdlBarrierLookup, DdlBarrierStore, InMemoryCheckpointStore};

#[path = "ack/cases.rs"]
mod ack_cases;

fn valid_outcome() -> TargetDdlApplyOutcome {
    TargetDdlApplyOutcome {
        barrier_id: "ddl-barrier-123".to_string(),
        applied_statements: 1,
        release_gate: "post_ddl_dml_release".to_string(),
        plan_sha256: "a".repeat(64),
        statement_sha256s: vec!["b".repeat(64)],
    }
}
