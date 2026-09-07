use std::collections::{HashMap, HashSet};

use trellara_protocol::{ChangeRecord, Operation, PartitionedScaleDecision, TransactionEnvelope};

use crate::plan_delete::plan_delete;
use crate::plan_insert::plan_insert;
use crate::plan_truncate::plan_truncate;
use crate::plan_update::plan_update;
use crate::sql::SqlStatement;
use crate::{ApplyError, ApplyTablePolicy, Result};

pub fn plan_envelope(envelope: &TransactionEnvelope) -> Result<Vec<SqlStatement>> {
    plan_envelope_with_policies(envelope, &HashMap::new())
}

pub fn plan_change(change: &ChangeRecord) -> Result<SqlStatement> {
    plan_change_with_policy(change, None)?.ok_or(ApplyError::NoMutableColumns {
        total_order: change.total_order,
    })
}

pub(crate) fn plan_envelope_with_policies(
    envelope: &TransactionEnvelope,
    table_policies: &HashMap<String, ApplyTablePolicy>,
) -> Result<Vec<SqlStatement>> {
    envelope.validate()?;
    let partitioned_scale_readiness = envelope.partitioned_scale_readiness();
    match partitioned_scale_readiness.decision {
        PartitionedScaleDecision::PartitionParallelDml
        | PartitionedScaleDecision::EmptyTransaction => {}
        PartitionedScaleDecision::DdlBarrierRequired => {
            return Err(ApplyError::DdlBarrierRequired {
                transaction_id: envelope.transaction_id.clone(),
                boundary_kind: partitioned_scale_readiness.boundary_kind,
                partitioned_scale_decision: partitioned_scale_readiness.decision,
                reason: partitioned_scale_readiness.reason.to_string(),
            });
        }
    }
    require_schema_version_evidence(envelope)?;
    let mut statements = Vec::new();
    for change in &envelope.changes {
        let policy = change
            .relation
            .as_ref()
            .and_then(|relation| table_policies.get(&relation.display_name()));
        if let Some(statement) = plan_change_with_policy(change, policy)? {
            statements.push(statement);
        }
    }
    statements.sort_by_key(|statement| statement.total_order);
    Ok(statements)
}

fn require_schema_version_evidence(envelope: &TransactionEnvelope) -> Result<()> {
    let schema_relations = envelope
        .schema_versions
        .iter()
        .filter_map(|schema_version| schema_version.relation.as_ref())
        .map(|relation| relation.display_name())
        .collect::<HashSet<_>>();

    for change in &envelope.changes {
        let relation = change
            .relation
            .as_ref()
            .ok_or(ApplyError::MissingRelation {
                total_order: change.total_order,
            })?;
        let relation_name = relation.display_name();
        if !schema_relations.contains(&relation_name) {
            return Err(ApplyError::MissingSchemaVersionEvidence {
                total_order: change.total_order,
                relation: relation_name,
            });
        }
    }
    Ok(())
}

fn plan_change_with_policy(
    change: &ChangeRecord,
    policy: Option<&ApplyTablePolicy>,
) -> Result<Option<SqlStatement>> {
    let operation = Operation::try_from(change.operation)
        .map_err(|_| ApplyError::UnsupportedOperation(change.operation))?;

    match operation {
        Operation::Insert => plan_insert(change, policy).map(Some),
        Operation::Update => plan_update(change, policy),
        Operation::Delete => plan_delete(change).map(Some),
        Operation::Truncate => plan_truncate(change).map(Some),
        Operation::Unspecified => Err(ApplyError::UnsupportedOperation(change.operation)),
    }
}
