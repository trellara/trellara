use crate::*;

pub(crate) fn execute_stream_command(command: StreamCommand) -> Result<String> {
    match command {
        StreamCommand::InspectLocal(args) => {
            let config = TrellaraConfig::from_path(&args.config)?;
            config.validate()?;
            Ok(serde_json::to_string_pretty(
                &inspect_configured_local_stream(&config)?,
            )?)
        }
        StreamCommand::LocateLocal(args) => {
            let config = TrellaraConfig::from_path(&args.config)?;
            config.validate()?;
            Ok(serde_json::to_string_pretty(
                &locate_configured_local_stream_transaction(&config, &args)?,
            )?)
        }
        StreamCommand::ReconstructLocal(args) => {
            let config = TrellaraConfig::from_path(&args.config)?;
            config.validate()?;
            Ok(serde_json::to_string_pretty(
                &reconstruct_configured_local_stream_transaction(&config, &args)?,
            )?)
        }
        StreamCommand::SeekLocal(args) => {
            let config = TrellaraConfig::from_path(&args.config)?;
            config.validate()?;
            Ok(serde_json::to_string_pretty(
                &seek_configured_local_stream(&config, &args)?,
            )?)
        }
    }
}
