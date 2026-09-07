use crate::{ContractCheck, ContractSeverity, SourceCaptureKind, TrellaraConfig};

pub(crate) fn pgoutput_relation_metadata_contract_checks(
    config: &TrellaraConfig,
    table: &trellara_pg_capture::TablePreflight,
) -> Vec<ContractCheck> {
    if config.source.capture != SourceCaptureKind::PgOutput {
        return Vec::new();
    }

    let relation = table.qualified_name();
    let name = format!("pgoutput_relation_metadata:{relation}");
    match table.schema_fingerprint {
        Some(fingerprint) => vec![ContractCheck::passed(
            name,
            format!(
                "{relation} contract is fed by live pgoutput relation metadata fingerprint {fingerprint}"
            ),
        )],
        None => vec![ContractCheck::passed_with_severity(
            name,
            ContractSeverity::Warning,
            format!(
                "{relation} did not expose a pgoutput relation metadata fingerprint; pinning and drift checks cannot be proven from this preflight"
            ),
        )],
    }
}
