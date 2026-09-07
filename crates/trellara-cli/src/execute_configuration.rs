use std::fs;

use crate::{migrate_config_path, redact_config_yaml, CliError, ConfigurationCommand, Result};

pub(crate) fn execute_configuration_command(command: ConfigurationCommand) -> Result<String> {
    match command {
        ConfigurationCommand::Migrate(args) => Ok(migrate_config_path(&args.config)?.yaml),
        ConfigurationCommand::Redact(args) => {
            let label = args.config.display().to_string();
            let contents =
                fs::read_to_string(&args.config).map_err(|source| CliError::ReadConfig {
                    path: label.clone(),
                    source,
                })?;
            redact_config_yaml(&contents, &label)
        }
    }
}
