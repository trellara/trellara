use crate::contract_types::*;
use crate::{PartitionNullKeyPolicy, TrellaraConfig};

pub(crate) fn partitioned_transaction_contract_checks(
    config: &TrellaraConfig,
    tables: &[trellara_pg_capture::TablePreflight],
) -> Vec<ContractCheck> {
    let partition = config
        .dataset
        .partition
        .as_ref()
        .expect("validated partition settings");
    let mut checks = vec![ContractCheck::passed(
        "transaction_boundary:partition_manifest",
        format!(
            "partitioned scale mode emits manifest and commit marker barriers across {} partitions",
            partition.partition_count
        ),
    )];
    checks.push(ContractCheck::passed(
        "partition_policy:null_key",
        format!(
            "null partition keys use policy {}",
            partition.null_key_policy
        ),
    ));
    checks.push(ContractCheck::passed_with_severity(
        "partition_policy:key_change",
        partition.key_change_policy.contract_severity(),
        partition.key_change_policy.contract_message(),
    ));

    for table in tables {
        checks.push(partition_key_contract_check(config, table));
    }

    checks
}

fn partition_key_contract_check(
    config: &TrellaraConfig,
    table: &trellara_pg_capture::TablePreflight,
) -> ContractCheck {
    let partition = config
        .dataset
        .partition
        .as_ref()
        .expect("validated partition settings");
    let relation = table.qualified_name();
    match table
        .columns
        .iter()
        .find(|column| column.name == partition.key_column)
    {
        Some(column) if column.nullable => match partition.null_key_policy {
            PartitionNullKeyPolicy::RouteToSingletonPartition => {
                ContractCheck::passed_with_severity(
                    format!("partition_key:{relation}"),
                    ContractSeverity::Warning,
                    format!(
                        "{relation} partition key {} is nullable; null values route to singleton partition 0",
                        partition.key_column
                    ),
                )
            }
            PartitionNullKeyPolicy::RouteToDeadLetterPartition => {
                ContractCheck::passed_with_severity(
                    format!("partition_key:{relation}"),
                    ContractSeverity::Warning,
                    format!(
                        "{relation} partition key {} is nullable; null values route to the dead-letter partition",
                        partition.key_column
                    ),
                )
            }
            PartitionNullKeyPolicy::DeriveFromPrimaryKey => {
                if table.columns.iter().any(|column| column.is_key) {
                    ContractCheck::passed_with_severity(
                        format!("partition_key:{relation}"),
                        ContractSeverity::Warning,
                        format!(
                            "{relation} partition key {} is nullable; null values derive a stable route from primary-key columns",
                            partition.key_column
                        ),
                    )
                } else {
                    ContractCheck::failed(
                        format!("partition_key:{relation}"),
                        ContractSeverity::Error,
                        format!(
                            "{relation} partition key {} is nullable but no primary key is available for null_key_policy=derive_from_primary_key",
                            partition.key_column
                        ),
                        "add a primary key, choose a non-null ownership key, or choose null_key_policy=route_to_singleton_partition or route_to_dead_letter_partition",
                    )
                }
            }
            PartitionNullKeyPolicy::Quarantine => ContractCheck::failed(
                format!("partition_key:{relation}"),
                ContractSeverity::Error,
                format!(
                    "{relation} partition key {} is nullable",
                    partition.key_column
                ),
                format!(
                    "make the partition key non-null, choose a non-null business ownership key, or set null_key_policy=route_to_singleton_partition when a deliberate hot lane is acceptable; current policy is {}",
                    partition.null_key_policy
                ),
            ),
        },
        Some(column) if !column.is_key => ContractCheck::failed(
            format!("partition_key:{relation}"),
            ContractSeverity::Warning,
            format!(
                "{relation} partition key {} is not part of the primary key, so key updates can move transaction lanes",
                partition.key_column
            ),
            format!(
                "prefer an immutable primary/business key or enforce key_change_policy={} in writers",
                partition.key_change_policy
            ),
        ),
        Some(_) => ContractCheck::passed(
            format!("partition_key:{relation}"),
            format!(
                "{relation} partition key {} is present, non-null, and key-backed",
                partition.key_column
            ),
        ),
        None => ContractCheck::failed(
            format!("partition_key:{relation}"),
            ContractSeverity::Error,
            format!(
                "{relation} is missing partition key column {}",
                partition.key_column
            ),
            "add the partition key to every table in the flow or choose a different key",
        ),
    }
}
