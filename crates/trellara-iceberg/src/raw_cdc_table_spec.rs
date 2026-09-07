use std::collections::{BTreeMap, BTreeSet};

use trellara_lake::LakeRawCdcEpochWritePlan;

use crate::planner_mappings::validated_mappings;
use crate::raw_cdc_schema::{raw_cdc_columns, raw_cdc_partition_fields, schema_fingerprint};
use crate::{IcebergCommitConfig, IcebergIntegrationError, IcebergRawCdcTableSpec, Result};

pub fn plan_raw_cdc_iceberg_table_specs(
    write_plan: &LakeRawCdcEpochWritePlan,
    config: &IcebergCommitConfig,
) -> Result<Vec<IcebergRawCdcTableSpec>> {
    let mappings = validated_mappings(config)?;
    let mut tables = BTreeMap::<String, RawCdcTableSpecSeed>::new();

    for file in &write_plan.data_files {
        let target = mappings.get(&file.table_name).ok_or_else(|| {
            IcebergIntegrationError::MissingTableMapping {
                lake_table_name: file.table_name.clone(),
            }
        })?;
        let seed = tables
            .entry(file.table_name.clone())
            .or_insert_with(|| RawCdcTableSpecSeed {
                lake_table_name: file.table_name.clone(),
                relation: file.relation.clone(),
                target: target.clone(),
                source_buckets: BTreeSet::new(),
            });
        if seed.relation != file.relation {
            return Err(IcebergIntegrationError::WritePlanBoundaryMismatch {
                field: "raw_cdc_table_relation",
                expected: seed.relation.clone(),
                actual: file.relation.clone(),
            });
        }
        seed.source_buckets.insert(file.source_bucket);
    }

    tables
        .into_values()
        .map(|seed| {
            let columns = raw_cdc_columns();
            let partition_fields = raw_cdc_partition_fields();
            Ok(IcebergRawCdcTableSpec {
                lake_table_name: seed.lake_table_name,
                relation: seed.relation,
                target: seed.target,
                schema_fingerprint_sha256: schema_fingerprint(&columns, &partition_fields),
                columns,
                partition_fields,
                source_bucket_count: seed.source_buckets.len(),
                source_buckets: seed.source_buckets.into_iter().collect(),
            })
        })
        .collect()
}

struct RawCdcTableSpecSeed {
    lake_table_name: String,
    relation: String,
    target: crate::IcebergTableIdentifier,
    source_buckets: BTreeSet<u32>,
}
