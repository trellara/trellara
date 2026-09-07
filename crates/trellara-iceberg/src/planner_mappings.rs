use std::collections::{BTreeMap, BTreeSet};

use crate::{IcebergCommitConfig, IcebergIntegrationError, IcebergTableIdentifier, Result};

pub(crate) fn validated_mappings(
    config: &IcebergCommitConfig,
) -> Result<BTreeMap<String, IcebergTableIdentifier>> {
    let mut mappings = BTreeMap::new();
    let mut targets = BTreeSet::new();
    for mapping in &config.table_mappings {
        if mapping.lake_table_name.is_empty()
            || mapping.lake_table_name.trim() != mapping.lake_table_name
        {
            return Err(IcebergIntegrationError::MissingTableMapping {
                lake_table_name: mapping.lake_table_name.clone(),
            });
        }
        mapping.target.validate()?;
        if mappings
            .insert(mapping.lake_table_name.clone(), mapping.target.clone())
            .is_some()
        {
            return Err(IcebergIntegrationError::DuplicateTableMapping {
                lake_table_name: mapping.lake_table_name.clone(),
            });
        }
        let qualified_target = mapping.target.qualified_name();
        if !targets.insert(qualified_target.clone()) {
            return Err(IcebergIntegrationError::DuplicateIcebergTarget {
                target: qualified_target,
            });
        }
    }
    Ok(mappings)
}
