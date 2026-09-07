use super::*;

mod ack;
mod envelope_apply;
mod envelope_boundary;
mod evidence;
mod plan;
mod protocol;
mod protocol_barrier;
mod protocol_plan;
mod release;

fn safe_plan() -> TargetDdlApplyPlan {
    TargetDdlApplyPlan {
        barrier_id: "ddl-barrier-123".to_string(),
        transaction_boundary_rule: TARGET_DDL_TRANSACTION_BOUNDARY.to_string(),
        blockers: Vec::new(),
        release_gate: "post_ddl_dml_release".to_string(),
        statements: vec![TargetDdlStatement {
            change: "add_nullable_column:public.sales.discount_code:text".to_string(),
            sql:
                "ALTER TABLE \"public\".\"sales\" ADD COLUMN IF NOT EXISTS \"discount_code\" text;"
                    .to_string(),
        }],
    }
}

fn safe_evidence() -> TargetDdlApplyPlanEvidence {
    TargetDdlApplyPlanEvidence {
        barrier_id: "ddl-barrier-123".to_string(),
        executable: true,
        transaction_boundary_rule: TARGET_DDL_TRANSACTION_BOUNDARY.to_string(),
        blockers: Vec::new(),
        statements: vec![TargetDdlStatementEvidence {
            change: "add_nullable_column:public.sales.discount_code:text".to_string(),
            sink: "target_postgres".to_string(),
            sql:
                "ALTER TABLE \"public\".\"sales\" ADD COLUMN IF NOT EXISTS \"discount_code\" text;"
                    .to_string(),
            transaction_scope: "single target schema transaction before barrier ACK".to_string(),
            release_gate_code: "post_ddl_dml_release".to_string(),
        }],
    }
}

pub(super) fn ddl_envelope() -> TransactionEnvelope {
    let mut envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-ddl".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: Vec::new(),
    });
    envelope.ddl_events = vec![DdlEvent::additive_column(
        "tx-ddl",
        1,
        relation(),
        "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"discount_code\" text;",
        12_345,
        67_890,
    )];
    envelope.finalize_checksum();
    envelope
}
