use std::collections::HashSet;

use crate::{
    ddl_change_boundary_rule, ddl_change_compatibility, ddl_change_decision, ddl_change_relation,
    ddl_target_postgres_sql, parse_ddl_plan_change, DdlPlanApplyMode, DdlPlanChange, Result,
    TrellaraConfig,
};

pub(crate) fn ddl_plan_change(
    config: &TrellaraConfig,
    configured_relations: &HashSet<String>,
    apply_mode: DdlPlanApplyMode,
    input: &str,
) -> Result<DdlPlanChange> {
    let (kind, object) = parse_ddl_plan_change(input)?;
    let relation = ddl_change_relation(kind, &object)?;
    let configured_relation = configured_relations.contains(&relation);
    let (compatibility, reason) = ddl_change_compatibility(config, kind, configured_relation);
    let decision = ddl_change_decision(apply_mode, compatibility);
    let boundary_rule = ddl_change_boundary_rule(kind);
    let target_postgres_sql = ddl_target_postgres_sql(kind, &object, decision);

    Ok(DdlPlanChange {
        input: input.to_string(),
        kind,
        object,
        relation: Some(relation),
        configured_relation,
        compatibility,
        decision,
        reason,
        boundary_rule: boundary_rule.to_string(),
        target_postgres_sql,
        propagation_actions: Vec::new(),
    })
}
