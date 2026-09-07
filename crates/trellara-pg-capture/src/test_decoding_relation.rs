use trellara_protocol::RelationId;

use crate::{CaptureError, Result};

pub(crate) fn parse_single_relation_name(name: &str) -> Result<RelationId> {
    let relations = parse_relation_names(name)?;
    if relations.len() != 1 {
        return Err(CaptureError::TestDecodingParse(format!(
            "expected one relation name, got {name:?}"
        )));
    }
    Ok(relations
        .into_iter()
        .next()
        .expect("checked relation count"))
}

pub(crate) fn parse_relation_names(names: &str) -> Result<Vec<RelationId>> {
    let relations = names
        .split(", ")
        .map(parse_relation_name)
        .collect::<Result<Vec<_>>>()?;
    if relations.is_empty() {
        Err(CaptureError::TestDecodingParse(
            "expected at least one relation name".to_string(),
        ))
    } else {
        Ok(relations)
    }
}

fn parse_relation_name(name: &str) -> Result<RelationId> {
    let (schema, table) = name.split_once('.').ok_or_else(|| {
        CaptureError::TestDecodingParse(format!("relation name is not schema-qualified: {name:?}"))
    })?;
    validate_relation_component("schema", schema, name)?;
    validate_relation_component("table", table, name)?;
    if table.contains('.') {
        return Err(CaptureError::TestDecodingParse(format!(
            "relation name must be exactly schema.table: {name:?}"
        )));
    }
    Ok(RelationId::new(0, schema, table))
}

fn validate_relation_component(field: &str, component: &str, relation_name: &str) -> Result<()> {
    if component.is_empty() {
        return Err(CaptureError::TestDecodingParse(format!(
            "relation {field} is empty in {relation_name:?}"
        )));
    }
    if component != component.trim() {
        return Err(CaptureError::TestDecodingParse(format!(
            "relation {field} contains surrounding whitespace in {relation_name:?}"
        )));
    }
    if component.contains(',') {
        return Err(CaptureError::TestDecodingParse(format!(
            "relation {field} contains an ambiguous separator in {relation_name:?}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_schema_qualified_relation_name() {
        assert_eq!(
            parse_single_relation_name("public.sales").expect("relation"),
            RelationId::new(0, "public", "sales")
        );
    }

    #[test]
    fn parses_multi_relation_list_in_source_order() {
        assert_eq!(
            parse_relation_names("public.sales, audit.sales_archive").expect("relations"),
            vec![
                RelationId::new(0, "public", "sales"),
                RelationId::new(0, "audit", "sales_archive"),
            ]
        );
    }

    #[test]
    fn rejects_unqualified_relation_name() {
        assert!(matches!(
            parse_relation_names("sales"),
            Err(CaptureError::TestDecodingParse(message))
                if message.contains("relation name is not schema-qualified")
        ));
    }

    #[test]
    fn rejects_empty_or_padded_relation_components() {
        for name in [".sales", "public.", " public.sales", "public.sales "] {
            assert!(
                matches!(
                    parse_relation_names(name),
                    Err(CaptureError::TestDecodingParse(message))
                        if message.contains("relation ")
                ),
                "expected {name:?} to be rejected"
            );
        }
    }

    #[test]
    fn rejects_ambiguous_relation_separators() {
        for name in ["public.sales,audit.sales_archive", "public.sales.audit"] {
            assert!(
                matches!(
                    parse_relation_names(name),
                    Err(CaptureError::TestDecodingParse(message))
                        if message.contains("ambiguous separator")
                            || message.contains("exactly schema.table")
                ),
                "expected {name:?} to be rejected"
            );
        }
    }

    #[test]
    fn rejects_multiple_relations_when_single_required() {
        assert!(matches!(
            parse_single_relation_name("public.sales, public.sale_items"),
            Err(CaptureError::TestDecodingParse(message))
                if message.contains("expected one relation name")
        ));
    }
}
