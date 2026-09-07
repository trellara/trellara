use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};

use crate::{LakeError, LakeStragglerPolicy};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LakeEpochSourceWindow {
    pub source_id: String,
    pub start_lsn: Option<String>,
    pub end_lsn: Option<String>,
}

impl LakeEpochSourceWindow {
    pub fn new(
        source_id: impl Into<String>,
        start_lsn: Option<impl Into<String>>,
        end_lsn: Option<impl Into<String>>,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            start_lsn: start_lsn.map(Into::into),
            end_lsn: end_lsn.map(Into::into),
        }
    }
}

pub fn deterministic_epoch_id(
    dataset_id: &str,
    required_sources: impl IntoIterator<Item = impl Into<String>>,
    straggler_policy: &LakeStragglerPolicy,
    source_windows: &[LakeEpochSourceWindow],
) -> Result<String, LakeError> {
    let required_sources = required_sources
        .into_iter()
        .map(Into::into)
        .collect::<BTreeSet<String>>();
    let mut windows = BTreeMap::<String, &LakeEpochSourceWindow>::new();
    for window in source_windows {
        if windows.insert(window.source_id.clone(), window).is_some() {
            return Err(LakeError::DuplicateEpochSourceWindow {
                source_id: window.source_id.clone(),
            });
        }
    }

    let mut manifest = String::new();
    push_field(&mut manifest, "dataset", dataset_id);
    push_field(
        &mut manifest,
        "policy",
        &straggler_policy_key(straggler_policy),
    );
    for source_id in required_sources.union(&windows.keys().cloned().collect()) {
        push_field(&mut manifest, "source", source_id);
        push_field(
            &mut manifest,
            "required",
            if required_sources.contains(source_id) {
                "true"
            } else {
                "false"
            },
        );
        if let Some(window) = windows.get(source_id) {
            push_optional_field(&mut manifest, "start_lsn", window.start_lsn.as_deref());
            push_optional_field(&mut manifest, "end_lsn", window.end_lsn.as_deref());
        } else {
            push_optional_field(&mut manifest, "start_lsn", None);
            push_optional_field(&mut manifest, "end_lsn", None);
        }
    }

    let digest = format!("{:x}", Sha256::digest(manifest.as_bytes()));
    Ok(format!(
        "epoch-{}-{}",
        epoch_id_component(dataset_id),
        &digest[..16]
    ))
}

fn straggler_policy_key(policy: &LakeStragglerPolicy) -> String {
    match policy {
        LakeStragglerPolicy::WaitAllRequired => "wait_all_required".to_string(),
        LakeStragglerPolicy::PublishWithGaps { grace_ms } => {
            format!("publish_with_gaps:{grace_ms}")
        }
        LakeStragglerPolicy::QuarantineOnGap => "quarantine_on_gap".to_string(),
    }
}

fn push_optional_field(manifest: &mut String, name: &str, value: Option<&str>) {
    push_field(manifest, name, value.unwrap_or("<none>"));
}

fn push_field(manifest: &mut String, name: &str, value: &str) {
    manifest.push_str(name);
    manifest.push(':');
    manifest.push_str(&value.len().to_string());
    manifest.push(':');
    manifest.push_str(value);
    manifest.push('\n');
}

fn epoch_id_component(dataset_id: &str) -> String {
    let mut component = String::with_capacity(dataset_id.len());
    for character in dataset_id.chars() {
        if character.is_ascii_alphanumeric() {
            component.push(character.to_ascii_lowercase());
        } else {
            component.push('-');
        }
    }
    let component = component.trim_matches('-').to_string();
    if component.is_empty() {
        "dataset".to_string()
    } else {
        component
    }
}
