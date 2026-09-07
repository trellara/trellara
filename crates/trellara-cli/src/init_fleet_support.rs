use std::fmt::Write as _;

use crate::{require_non_empty, CliError, FleetInitConfigSummary, FleetInitSummary, Result};

pub(crate) fn fleet_init_next_commands(configs: &[FleetInitConfigSummary]) -> Vec<String> {
    let mut commands = Vec::new();
    if !configs.is_empty() {
        commands.push(format!(
            "trellara fleet report {} --format text",
            configs
                .iter()
                .map(|config| format!("--config {}", config.config))
                .collect::<Vec<_>>()
                .join(" ")
        ));
        if let Some(first) = configs.first() {
            commands.extend([
                format!(
                    "trellara quickstart --config {} --check --format text",
                    first.config
                ),
                format!("trellara evaluate --config {} --format text", first.config),
                format!("trellara pilot-package --config {}", first.config),
            ]);
        }
    }
    commands
}

pub(crate) fn render_fleet_init_readme(summary: &FleetInitSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "# Trellara Fleet Init").expect("write string");
    writeln!(&mut output).expect("write string");
    writeln!(&mut output, "fleet: {}", summary.fleet_id).expect("write string");
    writeln!(&mut output, "flows: {}", summary.flow_count).expect("write string");
    writeln!(&mut output, "manifest: {}", summary.manifest).expect("write string");
    output.push_str("\n## Flow Configs\n\n");
    for config in &summary.configs {
        writeln!(
            &mut output,
            "- `{}`: dataset={} slot={} stream={}",
            config.config, config.dataset_id, config.source_slot, config.stream_path
        )
        .expect("write string");
    }
    output.push_str("\n## Next Commands\n\n");
    for command in &summary.next_commands {
        writeln!(&mut output, "- `{command}`").expect("write string");
    }
    output.push_str("\n## Review Rule\n\n");
    output.push_str("Run the fleet report before the first design-partner review, then evaluate and package the first flow before adding more destinations.\n");
    output
}

pub(crate) fn config_slug(value: &str) -> Result<String> {
    require_non_empty("config slug", value)?;
    let mut slug = String::new();
    let mut previous_was_separator = false;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            previous_was_separator = false;
        } else if matches!(character, '-' | '_' | '.') && !previous_was_separator {
            slug.push('-');
            previous_was_separator = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        return Err(CliError::InvalidConfig(format!(
            "value {value:?} does not contain any slug-safe characters"
        )));
    }
    Ok(slug)
}
