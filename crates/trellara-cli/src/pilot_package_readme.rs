use std::fmt::Write as _;
use std::path::Path;

use crate::{push_pilot_package_artifacts, push_pilot_package_first_commands, TrellaraConfig};

pub(crate) fn render_pilot_package_readme(config: &TrellaraConfig, config_path: &Path) -> String {
    let mut output = String::new();
    writeln!(&mut output, "# Trellara Pilot Package").expect("write string");
    writeln!(&mut output).expect("write string");
    writeln!(&mut output, "config: {}", config_path.display()).expect("write string");
    writeln!(&mut output, "source: {}", config.source.id).expect("write string");
    writeln!(&mut output, "dataset: {}", config.dataset.id).expect("write string");
    writeln!(&mut output, "mode: {}", config.status_mode()).expect("write string");
    writeln!(&mut output).expect("write string");
    push_pilot_package_artifacts(&mut output);
    push_pilot_package_first_commands(&mut output, config_path);
    output
}
