use std::fs;

use trellara_protocol::TransactionEnvelope;

use crate::*;

pub(crate) async fn source_safety_command(args: SourceSafetyArgs) -> Result<String> {
    let rendered = if let Some(config_path) = &args.config {
        let config = TrellaraConfig::from_path(config_path)?;
        config.validate()?;
        let status = flow_status(&config).await?;
        let summary = SourceSafetySummary::from_status(status);

        render_source_safety_summary(&summary, args.format)
    } else {
        let summary = source_safety_from_database(&args).await?;

        render_direct_source_safety_summary(&summary, args.format)
    }?;

    if let Some(output) = &args.output {
        fs::write(output, &rendered).map_err(|source| CliError::WriteOutput {
            path: output.display().to_string(),
            source,
        })?;
        return Ok(format!(
            "wrote source-safety {} report to {}",
            args.format,
            output.display()
        ));
    }

    Ok(rendered)
}

pub(crate) fn inspect_transaction_command(args: InspectTransactionArgs) -> Result<String> {
    let bytes = fs::read(&args.file).map_err(|source| CliError::ReadInput {
        path: args.file.display().to_string(),
        source,
    })?;
    let envelope = TransactionEnvelope::decode_checked(&bytes)?;
    let summary = TransactionInspectSummary::from_envelope(&envelope);

    render_transaction_inspect_summary(&summary, args.format)
}
