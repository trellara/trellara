use sha2::{Digest, Sha256};
use trellara_protocol::DDL_PROPAGATION_CDC_BOUNDARY;

use crate::{
    ddl_propagation_policy_mode_label, DatasetMode, DdlPropagationPolicySummary, TrellaraConfig,
};

pub(crate) fn ddl_cdc_transaction_boundary(
    config: &TrellaraConfig,
    policy_modes: &[DdlPropagationPolicySummary],
) -> String {
    let policy_digest = ddl_policy_modes_sha256(policy_modes);
    let decisions = ddl_policy_decision_tokens(policy_modes).join(",");
    if config.dataset.mode == DatasetMode::PartitionedScaleMode {
        format!(
            "source commit LSN is the DDL barrier; propagation_boundary={DDL_PROPAGATION_CDC_BOUNDARY}; propagation_decisions={decisions}; propagation_policy_sha256={policy_digest}; DDL and DML share source transaction order; post-DDL DML stays invisible until every sink ACK and every partition lane watermark is at or beyond barrier_lsn"
        )
    } else {
        format!(
            "source commit LSN is the DDL barrier; propagation_boundary={DDL_PROPAGATION_CDC_BOUNDARY}; propagation_decisions={decisions}; propagation_policy_sha256={policy_digest}; DDL and DML share source transaction order; post-DDL DML stays invisible until every required sink ACK is at or beyond barrier_lsn"
        )
    }
}

fn ddl_policy_decision_tokens(policy_modes: &[DdlPropagationPolicySummary]) -> Vec<String> {
    vec![
        format!(
            "auto_apply:{}",
            policy_change_count(policy_modes, "auto_apply")
        ),
        format!(
            "manual_review:{}",
            policy_change_count(policy_modes, "staged_rollout")
                + policy_change_count(policy_modes, "manual_approval_required")
        ),
        format!(
            "unsupported:{}",
            policy_change_count(policy_modes, "block_unsupported")
        ),
        format!(
            "target_ack_required:{}",
            policy_change_count(policy_modes, "auto_apply")
                + policy_change_count(policy_modes, "staged_rollout")
                + policy_change_count(policy_modes, "manual_approval_required")
        ),
    ]
}

fn policy_change_count(policy_modes: &[DdlPropagationPolicySummary], mode_label: &str) -> usize {
    policy_modes
        .iter()
        .find(|policy| ddl_propagation_policy_mode_label(policy.mode) == mode_label)
        .map_or(0, |policy| policy.change_count)
}

fn ddl_policy_modes_sha256(policy_modes: &[DdlPropagationPolicySummary]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(DDL_PROPAGATION_CDC_BOUNDARY.as_bytes());
    for policy in policy_modes {
        hasher.update(b"\n");
        hasher.update(ddl_propagation_policy_mode_label(policy.mode).as_bytes());
        let activity = if policy.active {
            &b"|active|"[..]
        } else {
            &b"|inactive|"[..]
        };
        hasher.update(activity);
        hasher.update(policy.change_count.to_string().as_bytes());
        hasher.update(b"|");
        hasher.update(policy.release_rule.as_bytes());
        hasher.update(b"|");
        hasher.update(policy.approval_evidence.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}
