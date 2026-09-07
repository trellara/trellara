use crate::{
    source_failure_from_parts, target_failure_from_parts, FlowAlertSeverity, FlowFailureParts,
    FlowFailureSummary,
};

impl FlowFailureSummary {
    pub(crate) fn from_parts(parts: FlowFailureParts<'_>) -> Option<Self> {
        let recovery_action_codes = parts
            .recovery_actions
            .iter()
            .map(|action| action.code.clone())
            .collect::<Vec<_>>();

        if let Some(failure) = source_failure_from_parts(&parts, &recovery_action_codes) {
            return Some(failure);
        }
        if let Some(quarantine) = parts.latest_quarantine {
            return Some(Self::critical(
                "target_quarantine_blocked",
                format!(
                    "target quarantine contains transaction {} at LSN {}: {}",
                    quarantine.transaction_id, quarantine.commit_lsn, quarantine.reason
                ),
                Some(quarantine.last_seen_at.clone()),
                recovery_action_codes,
            ));
        }
        if let Some(failure) = target_failure_from_parts(&parts, &recovery_action_codes) {
            return Some(failure);
        }

        None
    }

    pub(crate) fn critical(
        code: impl Into<String>,
        message: impl Into<String>,
        occurred_at: Option<String>,
        recovery_action_codes: Vec<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: FlowAlertSeverity::Critical,
            message: message.into(),
            occurred_at,
            recovery_action_codes,
        }
    }

    pub(crate) fn warning(
        code: impl Into<String>,
        message: impl Into<String>,
        occurred_at: Option<String>,
        recovery_action_codes: Vec<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: FlowAlertSeverity::Warning,
            message: message.into(),
            occurred_at,
            recovery_action_codes,
        }
    }
}
