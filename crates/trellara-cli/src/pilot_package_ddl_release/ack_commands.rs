use crate::{
    pilot_package_ddl_release_ack_samples::{
        raw_cdc_metadata_table, raw_cdc_partition_metadata_table, SAMPLE_ACK_LSN,
        SAMPLE_LAKE_EPOCH_ID, SAMPLE_LAKE_MANIFEST_DIGEST, SAMPLE_SPARK_ACCEPTED_BY,
        SAMPLE_SPARK_TEMPLATE_DIGEST, SAMPLE_SPARK_VIEW_COUNT,
    },
    DdlApplyPlanSummary, DdlPlanSummary, TrellaraConfig,
};

pub(super) fn ddl_release_ack_command(
    config: &TrellaraConfig,
    plan: &DdlPlanSummary,
    apply_plan: &DdlApplyPlanSummary,
    sink: &str,
    barrier_lsn: &str,
    schema_version: &str,
) -> String {
    let base = format!(
        "trellara schema ddl-barrier ack --config trellara.yml --barrier-id {} --sink {sink} --ack-lsn {SAMPLE_ACK_LSN} --schema-version {schema_version}",
        plan.propagation.barrier_id
    );
    match sink {
        "target_postgres" => target_postgres_ack_command(&base, apply_plan, barrier_lsn),
        "raw_cdc_lake" => {
            format!(
                "{base} --epoch-id {SAMPLE_LAKE_EPOCH_ID} --metadata-table {} --partition-metadata-table {} --manifest-digest {SAMPLE_LAKE_MANIFEST_DIGEST}",
                raw_cdc_metadata_table(config),
                raw_cdc_partition_metadata_table(config)
            )
        }
        "spark_derived_views" => {
            format!("{base} --template-digest {SAMPLE_SPARK_TEMPLATE_DIGEST} --accepted-by {SAMPLE_SPARK_ACCEPTED_BY} --view-count {SAMPLE_SPARK_VIEW_COUNT}")
        }
        "partition_visibility" => partition_visibility_ack_command(config, &base, barrier_lsn),
        _ => format!("{base} --detail <{sink}-ack-evidence>"),
    }
}

fn target_postgres_ack_command(
    base: &str,
    apply_plan: &DdlApplyPlanSummary,
    barrier_lsn: &str,
) -> String {
    let statement_args = apply_plan
        .statements
        .iter()
        .map(|statement| format!(" --statement-sha256 {}", statement.statement_sha256))
        .collect::<String>();
    format!(
        "{base} --barrier-lsn {barrier_lsn} --plan-sha256 {}{statement_args}",
        apply_plan.plan_sha256
    )
}

fn partition_visibility_ack_command(
    config: &TrellaraConfig,
    base: &str,
    barrier_lsn: &str,
) -> String {
    let partition_count = config
        .dataset
        .partition
        .as_ref()
        .map(|partition| partition.partition_count)
        .unwrap_or(1);
    let partition_lsn_args = (0..partition_count)
        .map(|partition_id| {
            format!(
                " --partition-durable-lsn {partition_id}={SAMPLE_ACK_LSN} --partition-applied-lsn {partition_id}={SAMPLE_ACK_LSN}"
            )
        })
        .collect::<String>();
    format!(
        "{base} --barrier-lsn {barrier_lsn} --expected-partition-count {partition_count}{partition_lsn_args}"
    )
}
