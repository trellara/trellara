use crate::render_lake_spark_pyspark_runner;

use super::*;

#[test]
fn spark_template_render_replaces_parameters_and_preserves_epoch_guard() {
    let sql = render_lake_spark_template_sql(
        LakeSparkTemplateKind::CurrentState,
        LakeSparkTemplateRenderParams {
            catalog: "spark_catalog",
            namespace: "retail",
            raw_cdc_table: "retail__public__sales__raw_cdc",
            epochs_table: "retail__trellara__fanin___trellara_epochs",
            epoch_partitions_table: "retail__trellara__fanin___trellara_epoch_partitions",
            verification_table: "retail__trellara__fanin___trellara_verification",
            quarantine_table: "retail__trellara__fanin___trellara_quarantine",
            target_table: "retail__public__sales__current",
            epoch_id: "epoch-1",
            primary_key_column: "id",
            accept_complete_with_gaps: true,
            unsafe_allow_non_consumable_epoch: false,
            unsafe_override_reason: "",
        },
    );

    assert!(!sql.contains("${"));
    assert!(sql.contains("spark_catalog.retail.retail__public__sales__raw_cdc"));
    assert!(sql.contains("spark_catalog.retail.retail__trellara__fanin___trellara_epochs"));
    assert!(sql.contains("spark_catalog.retail.retail__trellara__fanin___trellara_verification"));
    assert!(sql.contains("epoch_id = 'epoch-1'"));
    assert!(sql.contains("checksum_status = 'match'"));
    assert!(sql.contains("complete_with_gaps"));
    assert!(sql.contains("true"));
    assert!(sql.contains("false = true"));
}

#[test]
fn pyspark_runner_refuses_unresolved_templates_and_records_epoch_choice() {
    let runner = render_lake_spark_pyspark_runner(
        LakeSparkTemplateKind::CurrentState,
        "spark-current-state.sql",
        LakeSparkTemplateRenderParams {
            catalog: "spark_catalog",
            namespace: "retail",
            raw_cdc_table: "raw",
            epochs_table: "epochs",
            epoch_partitions_table: "partitions",
            verification_table: "verification",
            quarantine_table: "quarantine",
            target_table: "current",
            epoch_id: "epoch-1",
            primary_key_column: "id",
            accept_complete_with_gaps: false,
            unsafe_allow_non_consumable_epoch: true,
            unsafe_override_reason: "incident replay window",
        },
    );

    assert!(runner.contains("SparkSession.builder"));
    assert!(runner.contains("hashlib.sha256(sql_text.encode(\"utf-8\")).hexdigest()"));
    assert!(runner.contains("refusing unresolved Trellara Spark template placeholders"));
    assert!(runner.contains("spark-current-state.sql"));
    assert!(runner.contains("template_sha256="));
    assert!(runner.contains("epoch_id=epoch-1"));
    assert!(runner.contains("accept_complete_with_gaps=false"));
    assert!(runner.contains("unsafe_allow_non_consumable_epoch=true"));
    assert!(runner.contains("unsafe_override_reason=incident replay window"));
    assert!(!runner.contains("${"));
}

#[test]
fn spark_template_digest_is_stable_for_rendered_sql() {
    let first = lake_spark_template_sha256("SELECT 1;\n");
    let second = lake_spark_template_sha256("SELECT 1;\n");
    let changed = lake_spark_template_sha256("SELECT 2;\n");

    assert_eq!(first, second);
    assert_eq!(first.len(), 64);
    assert!(first.chars().all(|character| character.is_ascii_hexdigit()));
    assert_ne!(first, changed);
}

#[test]
fn lake_epochs_table_uses_reserved_fanin_namespace() {
    assert_eq!(
        lake_epochs_table_name("retail"),
        "retail__trellara__fanin___trellara_epochs"
    );
}

#[test]
fn lake_epoch_partitions_table_uses_reserved_fanin_namespace() {
    assert_eq!(
        lake_epoch_partitions_table_name("retail"),
        "retail__trellara__fanin___trellara_epoch_partitions"
    );
}

#[test]
fn lake_verification_table_uses_reserved_fanin_namespace() {
    assert_eq!(
        lake_verification_table_name("retail"),
        "retail__trellara__fanin___trellara_verification"
    );
}

#[test]
fn lake_quarantine_table_uses_reserved_fanin_namespace() {
    assert_eq!(
        lake_quarantine_table_name("retail"),
        "retail__trellara__fanin___trellara_quarantine"
    );
}
