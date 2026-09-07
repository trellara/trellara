use crate::contract_types::*;
use crate::{partitioned_transaction_contract_checks, DatasetMode, TrellaraConfig};

pub(crate) fn row_filter_contract_checks(config: &TrellaraConfig) -> Vec<ContractCheck> {
    config
        .dataset
        .tables
        .iter()
        .filter_map(|table| {
            let row_filter = table.verify.as_ref()?.row_filter.as_deref()?;
            Some(ContractCheck::passed_with_severity(
                format!("row_filter:{}", table.relation_id().display_name()),
                ContractSeverity::Info,
                format!(
                    "{} validates and reseeds only rows matching ({row_filter})",
                    table.relation_id().display_name()
                ),
            ))
        })
        .collect()
}

pub(crate) fn transaction_boundary_contract_checks(
    config: &TrellaraConfig,
    tables: &[trellara_pg_capture::TablePreflight],
) -> Vec<ContractCheck> {
    match config.dataset.mode {
        DatasetMode::StrictTransactionOrder => strict_transaction_contract_checks(config),
        DatasetMode::PartitionedScaleMode => {
            partitioned_transaction_contract_checks(config, tables)
        }
    }
}

fn strict_transaction_contract_checks(config: &TrellaraConfig) -> Vec<ContractCheck> {
    if let Some(strict_chunking) = &config.dataset.strict_chunking {
        vec![ContractCheck::passed(
            "transaction_boundary:strict_chunk_manifest",
            format!(
                "strict transaction order chunks transactions above {} changes and applies only after the manifest and commit marker barrier proves every chunk is present",
                strict_chunking.max_changes_per_chunk
            ),
        )]
    } else {
        vec![ContractCheck::passed(
            "transaction_boundary:strict",
            "strict transaction order publishes and applies each source transaction as one envelope",
        )]
    }
}
