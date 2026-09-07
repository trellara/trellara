use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    IcebergIntegrationError, IcebergObjectMetadata, IcebergObjectStore,
    IcebergObjectStoreErrorKind, Result,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergOrphanCleanupRequest {
    pub prefix: String,
    pub now_ms: i64,
    pub minimum_age_ms: i64,
    pub referenced_object_keys: BTreeSet<String>,
    pub pending_object_keys: BTreeSet<String>,
    pub dry_run: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergOrphanCleanupReport {
    pub listed_count: usize,
    pub candidate_count: usize,
    pub deleted_object_keys: Vec<String>,
    pub dry_run_object_keys: Vec<String>,
    pub protected_object_keys: Vec<String>,
    pub changed_during_cleanup_object_keys: Vec<String>,
}

pub async fn cleanup_orphan_objects<S: IcebergObjectStore + ?Sized>(
    store: &S,
    request: &IcebergOrphanCleanupRequest,
) -> Result<IcebergOrphanCleanupReport> {
    validate_request(request)?;
    let cutoff = request.now_ms - request.minimum_age_ms;
    let listed = store
        .list(&request.prefix)
        .await
        .map_err(|error| object_error("list_orphan_candidates", &request.prefix, error))?;
    let mut report = empty_report(listed.len());
    for candidate in listed {
        if protected(request, &candidate, cutoff) {
            report.protected_object_keys.push(candidate.object_key);
            continue;
        }
        report.candidate_count += 1;
        if request.dry_run {
            report.dry_run_object_keys.push(candidate.object_key);
            continue;
        }
        let current = match store.head(&candidate.object_key).await {
            Ok(current) => current,
            Err(error) if error.kind == IcebergObjectStoreErrorKind::NotFound => continue,
            Err(error) => {
                return Err(object_error(
                    "revalidate_orphan_candidate",
                    &candidate.object_key,
                    error,
                ))
            }
        };
        if !same_object(&candidate, &current)
            || current
                .last_modified_ms
                .is_none_or(|modified| modified > cutoff)
        {
            report
                .changed_during_cleanup_object_keys
                .push(candidate.object_key);
            continue;
        }
        if let Err(delete_error) = store
            .delete(&candidate.object_key, current.version.as_deref())
            .await
        {
            if !matches!(store.head(&candidate.object_key).await, Err(error) if error.kind == IcebergObjectStoreErrorKind::NotFound)
            {
                return Err(object_error(
                    "delete_orphan_candidate",
                    &candidate.object_key,
                    delete_error,
                ));
            }
        }
        report.deleted_object_keys.push(candidate.object_key);
    }
    report.deleted_object_keys.sort();
    report.dry_run_object_keys.sort();
    report.protected_object_keys.sort();
    report.changed_during_cleanup_object_keys.sort();
    Ok(report)
}

fn validate_request(request: &IcebergOrphanCleanupRequest) -> Result<()> {
    let bad_prefix = request.prefix.is_empty()
        || request.prefix.starts_with('/')
        || !request.prefix.ends_with('/')
        || request.prefix.trim() != request.prefix
        || request
            .prefix
            .trim_end_matches('/')
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..");
    if bad_prefix || request.minimum_age_ms <= 0 || request.now_ms < request.minimum_age_ms {
        return Err(IcebergIntegrationError::UnsafeMaintenancePlan {
            target: request.prefix.clone(),
            reason: "cleanup prefix must be a non-empty normalized relative directory ending in '/', and minimum age/cutoff must be positive".to_string(),
        });
    }
    Ok(())
}

fn protected(
    request: &IcebergOrphanCleanupRequest,
    candidate: &IcebergObjectMetadata,
    cutoff: i64,
) -> bool {
    !candidate.object_key.ends_with(".parquet")
        || request
            .referenced_object_keys
            .contains(&candidate.object_key)
        || request.pending_object_keys.contains(&candidate.object_key)
        || candidate
            .last_modified_ms
            .is_none_or(|modified| modified > cutoff)
}

fn empty_report(listed_count: usize) -> IcebergOrphanCleanupReport {
    IcebergOrphanCleanupReport {
        listed_count,
        candidate_count: 0,
        deleted_object_keys: Vec::new(),
        dry_run_object_keys: Vec::new(),
        protected_object_keys: Vec::new(),
        changed_during_cleanup_object_keys: Vec::new(),
    }
}

fn same_object(left: &IcebergObjectMetadata, right: &IcebergObjectMetadata) -> bool {
    left.object_key == right.object_key
        && left.content_length == right.content_length
        && left.etag == right.etag
        && left.version == right.version
}

fn object_error(
    operation: &'static str,
    object_key: &str,
    error: impl std::fmt::Display,
) -> IcebergIntegrationError {
    IcebergIntegrationError::ObjectStore {
        operation,
        object_key: object_key.to_string(),
        message: error.to_string(),
    }
}
