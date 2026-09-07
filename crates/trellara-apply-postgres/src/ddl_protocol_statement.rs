use trellara_protocol::{
    classify_ddl_propagation, DdlEvent, DdlPropagationDisposition, TransactionEnvelope,
};

use crate::TargetDdlStatement;

pub(crate) fn ordered_ddl_events(envelope: &TransactionEnvelope) -> Vec<&DdlEvent> {
    let mut events = envelope.ddl_events.iter().collect::<Vec<_>>();
    events.sort_by_key(|event| event.total_order);
    events
}

pub(crate) fn collect_statement(
    event: &DdlEvent,
    blockers: &mut Vec<String>,
    statements: &mut Vec<TargetDdlStatement>,
) {
    let change = ddl_change_key(event);
    let Ok(decision) = classify_ddl_propagation(event) else {
        blockers.push(format!("{change} uses unsupported DDL operation"));
        return;
    };
    if decision.disposition != DdlPropagationDisposition::AutoApply {
        blockers.push(format!("{change} {}", decision.reason));
        return;
    }
    if event.release_gate != decision.release_gate {
        blockers.push(format!("{change} has invalid post-DDL release gate"));
        return;
    }
    if !statement_targets_event_relation(event) {
        blockers.push(format!(
            "{change} statement relation does not match DDL event relation"
        ));
        return;
    }
    statements.push(TargetDdlStatement {
        change,
        sql: event.statement.clone(),
    });
}

fn ddl_change_key(event: &DdlEvent) -> String {
    let relation = event
        .relation
        .as_ref()
        .map(|relation| relation.display_name())
        .unwrap_or_else(|| "<unknown>".to_string());
    format!("ddl:{}:{relation}:{}", event.total_order, event.statement)
}

fn statement_targets_event_relation(event: &DdlEvent) -> bool {
    let Some(relation) = &event.relation else {
        return false;
    };
    let Some(statement_relation) = alter_table_relation(&event.statement) else {
        return false;
    };
    normalize_relation(&statement_relation) == normalize_relation(&relation.display_name())
}

fn alter_table_relation(statement: &str) -> Option<String> {
    let statement = statement.trim();
    let lower = statement.to_ascii_lowercase();
    if !lower.starts_with("alter table ") {
        return None;
    }
    let after_alter_table = statement.get("alter table ".len()..)?.trim_start();
    let lower_after = after_alter_table.to_ascii_lowercase();
    let add_column_index = lower_after.find(" add column ")?;
    let relation = after_alter_table.get(..add_column_index)?.trim();
    if relation.is_empty() {
        None
    } else {
        Some(relation.to_string())
    }
}

fn normalize_relation(relation: &str) -> String {
    relation
        .split('.')
        .map(|part| part.trim().trim_matches('"').to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alter_table_relation_reads_quoted_schema_relation() {
        assert_eq!(
            alter_table_relation(
                "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"discount_code\" text;"
            )
            .as_deref(),
            Some("\"public\".\"sales\"")
        );
    }

    #[test]
    fn normalize_relation_matches_quoted_and_unquoted_names() {
        assert_eq!(
            normalize_relation("\"public\".\"sales\""),
            normalize_relation("public.sales")
        );
    }
}
