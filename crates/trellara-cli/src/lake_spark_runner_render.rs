use crate::{LakeSparkTemplateKind, LakeSparkTemplateRenderParams};

pub(crate) fn render_lake_spark_pyspark_runner(
    template: LakeSparkTemplateKind,
    sql_file_name: &str,
    params: LakeSparkTemplateRenderParams<'_>,
) -> String {
    let app_suffix = match template {
        LakeSparkTemplateKind::CurrentState => "current-state",
        LakeSparkTemplateKind::Scd2 => "scd2",
        LakeSparkTemplateKind::Maintenance => "maintenance",
        LakeSparkTemplateKind::Dashboard => "dashboard",
    };
    format!(
        r#"#!/usr/bin/env python3
import argparse
import hashlib
from pathlib import Path

from pyspark.sql import SparkSession


def sql_statements(sql_text):
    return [statement.strip() for statement in sql_text.split(";") if statement.strip()]


def main():
    parser = argparse.ArgumentParser(description="Run a Trellara verified-epoch Spark template.")
    parser.add_argument("--sql", default="{sql_file_name}")
    parser.add_argument("--app-name", default="trellara-{app_suffix}-{epoch_id}")
    args = parser.parse_args()

    sql_path = Path(args.sql)
    sql_text = sql_path.read_text(encoding="utf-8")
    template_sha256 = hashlib.sha256(sql_text.encode("utf-8")).hexdigest()
    unresolved_placeholder_prefix = "$" + "{{"
    if unresolved_placeholder_prefix in sql_text:
        raise SystemExit("refusing unresolved Trellara Spark template placeholders")

    spark = SparkSession.builder.appName(args.app_name).getOrCreate()
    try:
        for statement in sql_statements(sql_text):
            spark.sql(statement)
    finally:
        spark.stop()

    print("trellara_spark_template=ok")
    print("template={app_suffix}")
    print(f"template_sha256={{template_sha256}}")
    print("epoch_id={epoch_id}")
    print("target={catalog}.{namespace}.{target_table}")
    print("accept_complete_with_gaps={accept_complete_with_gaps}")
    print("unsafe_allow_non_consumable_epoch={unsafe_allow_non_consumable_epoch}")
    print("unsafe_override_reason={unsafe_override_reason}")


if __name__ == "__main__":
    main()
"#,
        accept_complete_with_gaps = params.accept_complete_with_gaps,
        catalog = params.catalog,
        epoch_id = params.epoch_id,
        namespace = params.namespace,
        target_table = params.target_table,
        unsafe_allow_non_consumable_epoch = params.unsafe_allow_non_consumable_epoch,
        unsafe_override_reason = params.unsafe_override_reason
    )
}
