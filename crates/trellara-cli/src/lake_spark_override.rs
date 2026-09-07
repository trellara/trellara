use crate::{CliError, LakeSparkTemplateArgs, Result};

pub(crate) fn lake_spark_unsafe_override_reason(args: &LakeSparkTemplateArgs) -> Result<String> {
    if !args.unsafe_allow_non_consumable_epoch {
        return Ok(String::new());
    }
    let Some(reason) = &args.unsafe_override_reason else {
        return Err(CliError::InvalidConfig(
            "lake spark-template --unsafe-allow-non-consumable-epoch requires --unsafe-override-reason".to_string(),
        ));
    };
    if reason.trim().is_empty() {
        return Err(CliError::InvalidConfig(
            "lake spark-template --unsafe-override-reason must not be empty".to_string(),
        ));
    }
    Ok(reason.trim().to_string())
}

pub(crate) fn lake_spark_visibility_rule(
    args: &LakeSparkTemplateArgs,
    unsafe_override_reason: &str,
) -> String {
    if args.unsafe_allow_non_consumable_epoch {
        return format!(
            "unsafe override enabled: Spark may read non-consumable or unverified epochs only for audited reason `{}`",
            unsafe_override_reason
        );
    }
    "consume only epochs whose state is complete, or complete_with_gaps when the job explicitly accepts gaps, and whose verification checksum_status is match".to_string()
}
