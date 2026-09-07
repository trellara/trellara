use std::path::Path;

use crate::{
    pilot_package_artifacts::write_pilot_package_artifact, PilotPackageArtifact,
    PilotPackageMaterials, Result,
};

pub(crate) fn write_platform_spark_artifacts(
    output: &Path,
    materials: &PilotPackageMaterials,
) -> Result<Vec<PilotPackageArtifact>> {
    Ok(vec![
        write_pilot_package_artifact(
            output,
            "spark-current-state.sql",
            "sql",
            "rendered Spark SQL template for deriving current-state from verified lake epochs",
            &materials.spark_current_state.sql,
        )?,
        write_pilot_package_artifact(
            output,
            "spark-current-state.py",
            "python",
            "PySpark runner for the verified-epoch current-state SQL template that prints template_sha256 for Spark-derived-view DDL ACK evidence",
            &materials.spark_current_state.pyspark_runner,
        )?,
        write_pilot_package_artifact(
            output,
            "spark-scd2.sql",
            "sql",
            "rendered Spark SQL template for deriving SCD2 history from verified lake epochs",
            &materials.spark_scd2.sql,
        )?,
        write_pilot_package_artifact(
            output,
            "spark-scd2.py",
            "python",
            "PySpark runner for the verified-epoch SCD2 SQL template that prints template_sha256 for Spark-derived-view DDL ACK evidence",
            &materials.spark_scd2.pyspark_runner,
        )?,
        write_pilot_package_artifact(
            output,
            "spark-maintenance.sql",
            "sql",
            "rendered Spark SQL template for compacting and expiring snapshots after verified lake epochs",
            &materials.spark_maintenance.sql,
        )?,
        write_pilot_package_artifact(
            output,
            "spark-maintenance.py",
            "python",
            "PySpark runner for verification-gated Spark table maintenance that prints template_sha256 for maintenance ACK review",
            &materials.spark_maintenance.pyspark_runner,
        )?,
        write_pilot_package_artifact(
            output,
            "spark-completeness-dashboard.sql",
            "sql",
            "rendered Spark SQL queries for epoch and source-level lake fan-in completeness review",
            &materials.spark_completeness_dashboard.sql,
        )?,
        write_pilot_package_artifact(
            output,
            "spark-completeness-dashboard.py",
            "python",
            "PySpark runner for verification-gated lake fan-in completeness dashboard queries that prints template_sha256 for completeness-dashboard ACK review",
            &materials.spark_completeness_dashboard.pyspark_runner,
        )?,
        write_pilot_package_artifact(
            output,
            "spark-golden-fixture.json",
            "json",
            "deterministic raw CDC fixture with expected Spark current-state and SCD2 golden outputs",
            &serde_json::to_string_pretty(&materials.spark_golden_fixture)?,
        )?,
    ])
}
