use super::*;

impl PilotPackageContents {
    pub(super) fn assert_spark_contents(&self) {
        assert_content_contains(
            "Spark current-state template",
            &self.spark_current_state,
            &[
                "MERGE INTO spark_catalog.retail_sales.retail_sales__public__sales__current",
                "retail_sales__public__sales__raw_cdc",
                "retail_sales__trellara__fanin___trellara_verification",
                "v.checksum_status = 'match'",
                "e.state = 'complete_with_gaps'",
                "e.policy = 'publish_with_gaps'",
                "true = true",
            ],
        );
        assert_no_unresolved_placeholder("Spark current-state template", &self.spark_current_state);
        assert_content_contains(
            "Spark current-state runner",
            &self.spark_current_state_runner,
            &[
                "SparkSession.builder",
                "spark-current-state.sql",
                "refusing unresolved Trellara Spark template placeholders",
                "template_sha256=",
                "accept_complete_with_gaps=true",
            ],
        );
        assert_no_unresolved_placeholder(
            "Spark current-state runner",
            &self.spark_current_state_runner,
        );
        assert_content_contains(
            "Spark SCD2 template",
            &self.spark_scd2,
            &[
                "MERGE INTO spark_catalog.retail_sales.retail_sales__public__sales__scd2",
                "WHEN NOT MATCHED THEN INSERT",
                "v.checksum_status = 'match'",
                "__trellara_valid_from",
            ],
        );
        assert_no_unresolved_placeholder("Spark SCD2 template", &self.spark_scd2);
        assert_content_contains(
            "Spark SCD2 runner",
            &self.spark_scd2_runner,
            &[
                "SparkSession.builder",
                "spark-scd2.sql",
                "template=scd2",
                "template_sha256=",
                "accept_complete_with_gaps=true",
            ],
        );
        assert_no_unresolved_placeholder("Spark SCD2 runner", &self.spark_scd2_runner);
        assert_content_contains(
            "Spark maintenance template",
            &self.spark_maintenance,
            &[
                "CALL spark_catalog.system.rewrite_data_files",
                "CALL spark_catalog.system.expire_snapshots",
                "retail_sales__trellara__fanin___trellara_epoch_partitions",
                "retail_sales__trellara__fanin___trellara_verification",
                "v.checksum_status = 'match'",
                "e.policy = 'publish_with_gaps'",
            ],
        );
        assert_no_unresolved_placeholder("Spark maintenance template", &self.spark_maintenance);
        assert_content_contains(
            "Spark maintenance runner",
            &self.spark_maintenance_runner,
            &[
                "SparkSession.builder",
                "spark-maintenance.sql",
                "template=maintenance",
                "template_sha256=",
                "accept_complete_with_gaps=true",
            ],
        );
        assert_no_unresolved_placeholder(
            "Spark maintenance runner",
            &self.spark_maintenance_runner,
        );
        assert_content_contains(
            "Spark completeness dashboard template",
            &self.spark_completeness_dashboard,
            &[
                "epoch_release_gate",
                "safe_to_publish",
                "retail_sales__trellara__fanin___trellara_epochs",
                "retail_sales__trellara__fanin___trellara_epoch_sources",
                "retail_sales__trellara__fanin___trellara_epoch_partitions",
                "retail_sales__trellara__fanin___trellara_verification",
                "partition_event_count",
                "v.checksum_status = 'match'",
                "e.state = 'complete_with_gaps'",
                "e.policy = 'publish_with_gaps'",
                "true = true",
            ],
        );
        assert_no_unresolved_placeholder(
            "Spark completeness dashboard template",
            &self.spark_completeness_dashboard,
        );
        assert_content_contains(
            "Spark completeness dashboard runner",
            &self.spark_completeness_dashboard_runner,
            &[
                "SparkSession.builder",
                "spark-completeness-dashboard.sql",
                "template=dashboard",
                "template_sha256=",
                "accept_complete_with_gaps=true",
            ],
        );
        assert_no_unresolved_placeholder(
            "Spark completeness dashboard runner",
            &self.spark_completeness_dashboard_runner,
        );
        assert_content_contains(
            "Spark golden fixture",
            &self.spark_golden_fixture,
            &[
                "\"description\": \"deterministic raw CDC fixture proving Spark current-state and SCD2 derivations\"",
                "\"state\": \"complete\"",
                "\"checksum_status\": \"match\"",
                "\"raw_cdc\"",
                "\"expected_current_state\"",
                "\"expected_scd2\"",
                "\"record_key\": \"sale-100\"",
                "\"record_key\": \"sale-200\"",
                "\"rerun_idempotency_rule\"",
                "__trellara_idempotency_key",
            ],
        );
    }
}

fn assert_no_unresolved_placeholder(label: &str, content: &str) {
    assert!(
        !content.contains("${"),
        "{label} contains unresolved placeholder"
    );
}
