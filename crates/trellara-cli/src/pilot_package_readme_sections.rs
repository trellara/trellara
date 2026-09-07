use std::fmt::Write as _;
use std::path::Path;

use crate::{
    pilot_package_first_commands, EVIDENCE_REGISTRY_INSTRUCTION, PILOT_PACKAGE_ARTIFACT_LINES,
};

pub(crate) fn push_pilot_package_artifacts(output: &mut String) {
    output.push_str("## Artifacts\n\n");
    for line in PILOT_PACKAGE_ARTIFACT_LINES {
        writeln!(output, "{line}").expect("write string");
    }
    writeln!(output, "{EVIDENCE_REGISTRY_INSTRUCTION}").expect("write string");
}

pub(crate) fn push_pilot_package_first_commands(output: &mut String, config_path: &Path) {
    output.push_str("\n## First Commands\n\n");
    for command in pilot_package_first_commands(config_path) {
        writeln!(output, "- `{command}`").expect("write string");
    }
}
