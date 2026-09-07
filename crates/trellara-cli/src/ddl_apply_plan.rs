use crate::{
    ddl_apply_plan_digest, ddl_statement_digest, DdlApplyPlanStatement, DdlApplyPlanSummary,
    DdlPlanArgs, DdlPlanDecision, DdlPlanSummary, DdlPlanVerdict, Result, TrellaraConfig,
};

const TARGET_DDL_TRANSACTION_BOUNDARY: &str =
    "single_target_schema_transaction_then_barrier_ack_before_post_ddl_dml_release";

impl DdlApplyPlanSummary {
    pub(crate) fn from_args(config: &TrellaraConfig, args: &DdlPlanArgs) -> Result<Self> {
        let plan = DdlPlanSummary::from_args(config, args)?;
        let mut blockers = ddl_apply_plan_blockers(config, &plan);
        let statements = ddl_apply_plan_statements(&plan, &mut blockers);
        let plan_sha256 = ddl_apply_plan_digest(&statements);
        let executable = blockers.is_empty() && !statements.is_empty();
        let target_postgres_transaction_script =
            executable.then(|| ddl_apply_transaction_script(&plan_sha256, &statements));
        let target_postgres_ack_commands = if executable {
            target_postgres_ack_commands(
                args,
                &plan.propagation.barrier_id,
                &plan_sha256,
                &statements,
            )
        } else {
            Vec::new()
        };
        let steps = ddl_apply_plan_steps(executable);

        Ok(Self {
            source_id: plan.source_id,
            dataset_id: plan.dataset_id,
            barrier_id: plan.propagation.barrier_id,
            executable,
            transaction_boundary_rule: TARGET_DDL_TRANSACTION_BOUNDARY.to_string(),
            plan_sha256,
            blocker_count: blockers.len(),
            blockers,
            statement_count: statements.len(),
            statements,
            target_postgres_transaction_script,
            target_postgres_ack_commands,
            steps,
            next_commands: plan.next_commands,
        })
    }
}

fn ddl_apply_plan_blockers(config: &TrellaraConfig, plan: &DdlPlanSummary) -> Vec<String> {
    let mut blockers = Vec::new();
    if config.target.is_none() {
        blockers.push("target.database_url is required for target Postgres DDL apply-plan".into());
    }
    if plan.verdict == DdlPlanVerdict::Blocked {
        blockers.push("DDL propagation plan is blocked; no target SQL may be staged".into());
    }
    blockers
}

fn ddl_apply_plan_statements(
    plan: &DdlPlanSummary,
    blockers: &mut Vec<String>,
) -> Vec<DdlApplyPlanStatement> {
    let mut statements = Vec::new();
    for change in &plan.changes {
        match (&change.target_postgres_sql, change.decision) {
            (Some(sql), DdlPlanDecision::AutoApply | DdlPlanDecision::StageThenApply) => {
                statements.push(DdlApplyPlanStatement {
                    change: change.input.clone(),
                    sink: "target_postgres".to_string(),
                    sql: sql.clone(),
                    statement_sha256: ddl_statement_digest(sql),
                    transaction_scope: "single target schema transaction before barrier ACK".into(),
                    release_gate_code: "post_ddl_dml_release".into(),
                    release_gate: "post_ddl_dml_release waits for ddl-barrier status".into(),
                });
            }
            _ => blockers.push(format!(
                "change {} has no safe target_postgres_sql for decision {:?}",
                change.input, change.decision
            )),
        }
    }
    statements
}

fn ddl_apply_transaction_script(plan_sha256: &str, statements: &[DdlApplyPlanStatement]) -> String {
    let mut script = String::new();
    script.push_str("-- Trellara target Postgres DDL transaction script\n");
    script.push_str(
        "-- Execute on the target Postgres instance only after reviewing schema-ddl-apply-plan.\n",
    );
    script.push_str(&format!("-- plan_sha256={plan_sha256}\n"));
    script.push_str("-- Do not record target_postgres ACK until target schema fingerprint verification passes.\n");
    script.push_str("BEGIN;\n");
    for statement in statements {
        script.push_str(&format!(
            "-- change={} statement_sha256={}\n{}\n",
            statement.change, statement.statement_sha256, statement.sql
        ));
    }
    script.push_str("COMMIT;\n");
    script
}

fn target_postgres_ack_commands(
    args: &DdlPlanArgs,
    barrier_id: &str,
    plan_sha256: &str,
    statements: &[DdlApplyPlanStatement],
) -> Vec<String> {
    let statement_args = statements
        .iter()
        .map(|statement| format!(" --statement-sha256 {}", statement.statement_sha256))
        .collect::<String>();
    vec![format!(
        "trellara schema ddl-barrier ack --config {} --barrier-id {barrier_id} --sink target_postgres --ack-lsn <target-schema-ack-lsn> --schema-version <schema-version> --plan-sha256 {plan_sha256}{statement_args}",
        args.config.display()
    )]
}

fn ddl_apply_plan_steps(executable: bool) -> Vec<String> {
    if !executable {
        return vec!["fix blockers and rerun schema ddl-apply-plan".to_string()];
    }
    vec![
        "run schema discover and contract test against the current source and target".to_string(),
        "open one target Postgres schema transaction and execute statements in order".to_string(),
        "commit the schema transaction only after target preflight matches".to_string(),
        "record target_postgres ACK with the generated plan_sha256 and statement_sha256 values"
            .to_string(),
        "record the DDL barrier at the source DDL LSN and schema version".to_string(),
        "ACK target_postgres only after the target schema fingerprint matches".to_string(),
        "release post-DDL DML only when ddl-barrier status has no release blockers".to_string(),
    ]
}
