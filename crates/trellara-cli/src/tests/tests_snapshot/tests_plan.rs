use super::*;

#[test]
fn default_snapshot_run_id_is_operator_readable() {
    let run_id = default_snapshot_run_id();

    assert!(run_id.starts_with("snapshot-"));
    assert!(run_id.len() > "snapshot-".len());
}

#[test]
fn snapshot_copy_plan_keeps_selected_tables_separate_from_handoff_boundary() {
    let config = TrellaraConfig::from_yaml(&multi_table_strict_yaml(), "test").expect("parse");

    let plan = SnapshotCopyPlan::from_tables(&config.dataset.tables, Some("public.sales"))
        .expect("snapshot plan");

    assert_eq!(plan.selected_tables.len(), 1);
    assert_eq!(
        plan.selected_tables[0].relation_id().display_name(),
        "public.sales"
    );
    assert_eq!(
        plan.handoff_relations,
        vec!["public.sales".to_string(), "public.payments".to_string()]
    );
}
