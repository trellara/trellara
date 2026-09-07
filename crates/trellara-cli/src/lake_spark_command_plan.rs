use std::fmt::Write as _;

use crate::{lake_spark_template_kind_label, LakeSparkTemplateArgs, LakeSparkTemplateKind};

pub(crate) fn lake_spark_template_next_commands(
    args: &LakeSparkTemplateArgs,
    template: LakeSparkTemplateKind,
    unsafe_override_reason: &str,
) -> Vec<String> {
    let mut command = format!(
        "trellara lake spark-template {} --config {}",
        lake_spark_template_kind_label(template),
        args.config.display()
    );
    if let Some(table) = &args.table {
        write!(&mut command, " --table {table}").expect("write string");
    }
    write!(&mut command, " --epoch-id {}", args.epoch_id).expect("write string");
    if args.accept_complete_with_gaps {
        command.push_str(" --accept-complete-with-gaps");
    }
    if args.unsafe_allow_non_consumable_epoch {
        command.push_str(" --unsafe-allow-non-consumable-epoch");
        if !unsafe_override_reason.is_empty() {
            write!(
                &mut command,
                " --unsafe-override-reason {}",
                shell_quote(unsafe_override_reason)
            )
            .expect("write string");
        }
    }
    command.push_str(" --format text");

    vec![
        command,
        "run trellara lake fanin verify before publishing this SQL output to downstream consumers"
            .to_string(),
    ]
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
