use std::{fs, path::PathBuf};

use trellara_protocol::TransactionEnvelope;

use crate::{CliError, LakeFaninRunArgs, LakeWriterPlanArgs, Result};

pub(crate) fn read_lake_fanin_run_envelopes(
    args: &LakeFaninRunArgs,
) -> Result<Vec<TransactionEnvelope>> {
    read_envelopes(&args.files)
}

pub(crate) fn read_lake_writer_envelopes(
    args: &LakeWriterPlanArgs,
) -> Result<Vec<TransactionEnvelope>> {
    read_envelopes(&args.files)
}

fn read_envelopes(files: &[PathBuf]) -> Result<Vec<TransactionEnvelope>> {
    files
        .iter()
        .map(|file| {
            let bytes = fs::read(file).map_err(|source| CliError::ReadInput {
                path: file.display().to_string(),
                source,
            })?;
            Ok(TransactionEnvelope::decode_checked(&bytes)?)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_envelopes_reports_the_missing_input_path() {
        let missing = std::env::temp_dir().join(format!(
            "trellara-missing-lake-envelope-{}-{}.pb",
            std::process::id(),
            unique_test_suffix()
        ));

        let error = read_envelopes(std::slice::from_ref(&missing))
            .expect_err("missing envelope should fail");

        match error {
            CliError::ReadInput { path, .. } => {
                assert_eq!(path, missing.display().to_string());
            }
            other => panic!("expected read input error, got {other:?}"),
        }
    }

    fn unique_test_suffix() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time after unix epoch")
            .as_nanos()
    }
}
