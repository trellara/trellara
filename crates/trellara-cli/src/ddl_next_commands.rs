use std::path::Path;

use crate::ddl_labels::ddl_apply_mode_cli_value;
use crate::ddl_types::{
    DdlPlanVerdict, DdlPropagationPlan, DdlPropagationSink, DdlPropagationSinkKind,
};
use crate::DdlPlanApplyMode;

pub(crate) fn ddl_plan_next_commands(
    verdict: DdlPlanVerdict,
    config_path: &Path,
    changes: &[String],
    apply_mode: DdlPlanApplyMode,
    propagation: &DdlPropagationPlan,
) -> Vec<String> {
    match verdict {
        DdlPlanVerdict::ReadyToApply => {
            ddl_barrier_next_commands(config_path, changes, apply_mode, propagation, false)
        }
        DdlPlanVerdict::RequiresManualReview => {
            ddl_barrier_next_commands(config_path, changes, apply_mode, propagation, true)
        }
        DdlPlanVerdict::Blocked => vec![
            "update dataset.tables, unknown_table_policy, or explicit schema mapping before propagation".to_string(),
            format!("trellara contract test --config {}", config_path.display()),
            format!("trellara repair-plan --config {}", config_path.display()),
        ],
    }
}

fn ddl_barrier_next_commands(
    config_path: &Path,
    changes: &[String],
    apply_mode: DdlPlanApplyMode,
    propagation: &DdlPropagationPlan,
    include_snapshot_refresh: bool,
) -> Vec<String> {
    let config = config_path.display().to_string();
    let mut commands = vec![
        format!("trellara schema discover --config {config}"),
        format!("trellara contract test --config {config}"),
    ];
    if include_snapshot_refresh {
        commands.push(format!("trellara snapshot --config {config}"));
    }
    let change_args = changes
        .iter()
        .map(|change| format!(" --change {change}"))
        .collect::<String>();
    commands.push(format!(
        "trellara schema ddl-barrier record --config {config}{change_args} --apply-mode {} --barrier-lsn <ddl-lsn> --schema-version <schema-version>",
        ddl_apply_mode_cli_value(apply_mode)
    ));
    commands.extend(
        propagation
            .sinks
            .iter()
            .flat_map(|sink| ddl_barrier_ack_commands(&config, &propagation.barrier_id, sink)),
    );
    commands.push(format!(
        "trellara schema ddl-barrier status --config {config} --barrier-id {}",
        propagation.barrier_id
    ));
    commands
}

fn ddl_barrier_ack_commands(
    config: &str,
    barrier_id: &str,
    sink: &DdlPropagationSink,
) -> Vec<String> {
    let ack_command = ddl_barrier_ack_command(config, barrier_id, sink);
    if sink.kind == DdlPropagationSinkKind::SparkDerivedView {
        let template_commands = ["current-state", "scd2"]
            .map(|template| {
                format!(
                    "trellara lake spark-template {template} --config {config} --table <schema.table> --epoch-id <lake-epoch-id> --accept-complete-with-gaps --format json"
                )
            })
            .into_iter();
        template_commands
            .chain(std::iter::once(ack_command))
            .collect()
    } else {
        vec![ack_command]
    }
}

fn ddl_barrier_ack_command(config: &str, barrier_id: &str, sink: &DdlPropagationSink) -> String {
    let base = format!(
        "trellara schema ddl-barrier ack --config {config} --barrier-id {barrier_id} --sink {} --ack-lsn <sink-ack-lsn> --schema-version <schema-version>",
        sink.name
    );
    match sink.kind {
        DdlPropagationSinkKind::RawCdcLake => {
            format!(
                "{base} --epoch-id <lake-epoch-id> --metadata-table <raw-cdc-epoch-metadata-table> --partition-metadata-table <raw-cdc-epoch-partitions-table> --manifest-digest <lake-epoch-manifest-sha256>"
            )
        }
        DdlPropagationSinkKind::SparkDerivedView => {
            format!("{base} --template-digest <spark-template-sha256-hex-from-template_sha256> --accepted-by <reviewer-or-automation> --view-count <derived-view-count>")
        }
        DdlPropagationSinkKind::PartitionVisibility => {
            format!("{base} --barrier-lsn <ddl-lsn> --expected-partition-count <partition-count> --partition-durable-lsn <partition-id>=<durable-lsn> --partition-applied-lsn <partition-id>=<applied-lsn>")
        }
        DdlPropagationSinkKind::TargetPostgres => {
            format!("{base} --plan-sha256 <target-ddl-plan-sha256> --statement-sha256 <target-ddl-statement-sha256>")
        }
    }
}
