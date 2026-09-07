use super::*;

#[test]
fn target_ddl_plan_from_evidence_matches_cli_apply_plan_shape() {
    let apply_plan = target_ddl_apply_plan_from_evidence(safe_evidence()).expect("evidence plan");
    let transaction_plan = plan_target_ddl_transaction(apply_plan).expect("ddl transaction plan");

    assert_eq!(transaction_plan.barrier_id, "ddl-barrier-123");
    assert_eq!(transaction_plan.release_gate, "post_ddl_dml_release");
    assert_eq!(transaction_plan.statement_count, 1);
    assert_eq!(
        transaction_plan.statements[0].sql,
        "ALTER TABLE \"public\".\"sales\" ADD COLUMN IF NOT EXISTS \"discount_code\" text;"
    );
}

#[test]
fn target_ddl_plan_from_evidence_refuses_missing_boundary_rule() {
    let mut evidence = safe_evidence();
    evidence.transaction_boundary_rule = "best_effort_schema_apply".to_string();

    assert!(matches!(
        target_ddl_apply_plan_from_evidence(evidence),
        Err(ApplyError::MissingDdlField {
            field: "transaction_boundary_rule"
        })
    ));
}

#[test]
fn target_ddl_plan_from_evidence_refuses_missing_barrier_id() {
    let mut evidence = safe_evidence();
    evidence.barrier_id = "  ".to_string();

    assert!(matches!(
        target_ddl_apply_plan_from_evidence(evidence),
        Err(ApplyError::MissingDdlField {
            field: "barrier_id"
        })
    ));
}

#[test]
fn target_ddl_plan_from_evidence_rejects_padded_barrier_id() {
    let mut evidence = safe_evidence();
    evidence.barrier_id = " ddl-barrier-123".to_string();

    assert!(matches!(
        target_ddl_apply_plan_from_evidence(evidence),
        Err(ApplyError::InvalidDdlField { field, reason })
            if field == "barrier_id" && reason == "must not contain surrounding whitespace"
    ));
}

#[test]
fn target_ddl_plan_from_evidence_refuses_empty_executable_statements() {
    let mut evidence = safe_evidence();
    evidence.statements.clear();

    assert!(matches!(
        target_ddl_apply_plan_from_evidence(evidence),
        Err(ApplyError::MissingDdlField {
            field: "statements"
        })
    ));
}

#[test]
fn target_ddl_plan_from_evidence_refuses_weak_statement_scope() {
    let mut evidence = safe_evidence();
    evidence.statements[0].transaction_scope = "separate autocommit statements".to_string();

    assert!(matches!(
        target_ddl_apply_plan_from_evidence(evidence),
        Err(ApplyError::UnsafeDdlStatement { reason, .. })
            if reason.contains("single target schema transaction")
    ));
}

#[test]
fn target_ddl_plan_from_evidence_refuses_non_target_sink() {
    let mut evidence = safe_evidence();
    evidence.statements[0].sink = "raw_cdc_lake".to_string();

    assert!(matches!(
        target_ddl_apply_plan_from_evidence(evidence),
        Err(ApplyError::UnsafeDdlStatement { reason, .. })
            if reason.contains("target_postgres")
    ));
}

#[test]
fn target_ddl_plan_from_evidence_refuses_missing_release_gate_code() {
    let mut evidence = safe_evidence();
    evidence.statements[0].release_gate_code = "manual_gate".to_string();

    assert!(matches!(
        target_ddl_apply_plan_from_evidence(evidence),
        Err(ApplyError::UnsafeDdlStatement { reason, .. })
            if reason.contains("post-DDL DML release")
    ));
}

#[test]
fn target_ddl_plan_from_evidence_requires_statement_boundary_fields() {
    for field in [
        "change",
        "sink",
        "sql",
        "transaction_scope",
        "release_gate_code",
    ] {
        let mut evidence = safe_evidence();
        match field {
            "change" => evidence.statements[0].change.clear(),
            "sink" => evidence.statements[0].sink.clear(),
            "sql" => evidence.statements[0].sql.clear(),
            "transaction_scope" => evidence.statements[0].transaction_scope.clear(),
            "release_gate_code" => evidence.statements[0].release_gate_code.clear(),
            _ => unreachable!("test fields are exhaustive"),
        }

        assert!(matches!(
            target_ddl_apply_plan_from_evidence(evidence),
            Err(ApplyError::MissingDdlField { field: actual }) if actual == field
        ));
    }
}

#[test]
fn target_ddl_plan_from_evidence_rejects_padded_statement_boundary_fields() {
    for field in [
        "change",
        "sink",
        "sql",
        "transaction_scope",
        "release_gate_code",
    ] {
        let mut evidence = safe_evidence();
        match field {
            "change" => evidence.statements[0].change = " add-column".to_string(),
            "sink" => evidence.statements[0].sink = "target_postgres ".to_string(),
            "sql" => {
                evidence.statements[0].sql =
                    "\nALTER TABLE public.sales ADD COLUMN discount_code text;".to_string();
            }
            "transaction_scope" => {
                evidence.statements[0].transaction_scope =
                    " single target schema transaction before barrier ACK".to_string();
            }
            "release_gate_code" => {
                evidence.statements[0].release_gate_code = "post_ddl_dml_release ".to_string();
            }
            _ => unreachable!("test fields are exhaustive"),
        }

        assert!(matches!(
            target_ddl_apply_plan_from_evidence(evidence),
            Err(ApplyError::InvalidDdlField { field: actual, reason })
                if actual == field && reason == "must not contain surrounding whitespace"
        ));
    }
}

#[test]
fn target_ddl_plan_from_evidence_refuses_contract_changing_add_column() {
    for sql in [
        "ALTER TABLE public.sales ADD COLUMN code text NOT NULL;",
        "ALTER TABLE public.sales ADD COLUMN code text DEFAULT 'new';",
    ] {
        let mut evidence = safe_evidence();
        evidence.statements[0].sql = sql.to_string();

        let apply_plan =
            target_ddl_apply_plan_from_evidence(evidence).expect("evidence apply plan");

        assert!(matches!(
            plan_target_ddl_transaction(apply_plan),
            Err(ApplyError::UnsafeDdlStatement { reason, .. })
                if reason.contains("contract-changing DDL")
        ));
    }
}

#[test]
fn target_ddl_plan_from_evidence_refuses_non_executable_summary() {
    let mut evidence = safe_evidence();
    evidence.executable = false;
    evidence
        .blockers
        .push("target.database_url is required".to_string());

    assert!(matches!(
        target_ddl_apply_plan_from_evidence(evidence),
        Err(ApplyError::DdlApplyBlocked { blockers, .. })
            if blockers == vec!["target.database_url is required"]
    ));
}
