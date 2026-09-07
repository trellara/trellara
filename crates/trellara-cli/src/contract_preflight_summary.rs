use crate::{CliError, Result};

pub(crate) fn fail_on_preflight_summary(
    tables: &[trellara_pg_capture::TablePreflight],
) -> Result<()> {
    let issues = tables
        .iter()
        .flat_map(|table| {
            table
                .issues
                .iter()
                .map(|issue| format!("{}: {issue}", table.qualified_name()))
        })
        .collect::<Vec<_>>();
    if issues.is_empty() {
        Ok(())
    } else {
        Err(CliError::InvalidConfig(format!(
            "preflight failed: {}",
            issues.join("; ")
        )))
    }
}
