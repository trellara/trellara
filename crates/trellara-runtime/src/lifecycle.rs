use serde::{Deserialize, Serialize};

use crate::{Result, RuntimeContractError};

pub const RUNTIME_CONTRACT_VERSION: u16 = 1;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeService {
    Relay,
    Applier,
    IcebergWriter,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimePhase {
    Starting,
    Running,
    Draining,
    Stopped,
    Failed,
}

impl RuntimePhase {
    const fn label(self) -> &'static str {
        match self {
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
            Self::Failed => "failed",
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeReadiness {
    NotReady,
    Ready,
    Degraded,
    Backpressured,
    Blocked,
}

impl RuntimeReadiness {
    const fn label(self) -> &'static str {
        match self {
            Self::NotReady => "not_ready",
            Self::Ready => "ready",
            Self::Degraded => "degraded",
            Self::Backpressured => "backpressured",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuntimeHealth {
    pub contract_version: u16,
    pub service: RuntimeService,
    pub source_id: String,
    pub dataset_id: String,
    pub phase: RuntimePhase,
    pub readiness: RuntimeReadiness,
    pub accepting_work: bool,
    pub pending_work: u64,
    pub reason_code: Option<String>,
}

impl RuntimeHealth {
    #[must_use]
    pub fn starting(
        service: RuntimeService,
        source_id: impl Into<String>,
        dataset_id: impl Into<String>,
    ) -> Self {
        Self {
            contract_version: RUNTIME_CONTRACT_VERSION,
            service,
            source_id: source_id.into(),
            dataset_id: dataset_id.into(),
            phase: RuntimePhase::Starting,
            readiness: RuntimeReadiness::NotReady,
            accepting_work: false,
            pending_work: 0,
            reason_code: None,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.contract_version != RUNTIME_CONTRACT_VERSION {
            return Err(RuntimeContractError::UnsupportedVersion {
                expected: RUNTIME_CONTRACT_VERSION,
                actual: self.contract_version,
            });
        }
        require_identity("source_id", &self.source_id)?;
        require_identity("dataset_id", &self.dataset_id)?;
        validate_reason_code(self)?;
        validate_state(self)
    }
}

fn validate_reason_code(health: &RuntimeHealth) -> Result<()> {
    if let Some(reason_code) = &health.reason_code {
        let valid = !reason_code.is_empty()
            && reason_code
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');
        if !valid {
            return Err(RuntimeContractError::InvalidField {
                field: "reason_code",
                reason: "must contain only lowercase ASCII letters, digits, or underscores",
            });
        }
    }
    if matches!(
        health.readiness,
        RuntimeReadiness::Degraded | RuntimeReadiness::Backpressured | RuntimeReadiness::Blocked
    ) && health.reason_code.is_none()
    {
        return Err(RuntimeContractError::InvalidField {
            field: "reason_code",
            reason: "is required for degraded, backpressured, or blocked readiness",
        });
    }
    Ok(())
}

fn require_identity(field: &'static str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        Err(RuntimeContractError::InvalidField {
            field,
            reason: "cannot be blank",
        })
    } else {
        Ok(())
    }
}

fn validate_state(health: &RuntimeHealth) -> Result<()> {
    let valid = match health.phase {
        RuntimePhase::Starting | RuntimePhase::Draining | RuntimePhase::Stopped => {
            health.readiness == RuntimeReadiness::NotReady && !health.accepting_work
        }
        RuntimePhase::Failed => {
            health.readiness == RuntimeReadiness::Blocked && !health.accepting_work
        }
        RuntimePhase::Running => match health.readiness {
            RuntimeReadiness::Ready | RuntimeReadiness::Degraded => health.accepting_work,
            RuntimeReadiness::Backpressured | RuntimeReadiness::Blocked => !health.accepting_work,
            RuntimeReadiness::NotReady => false,
        },
    };
    if valid {
        Ok(())
    } else {
        Err(RuntimeContractError::InvalidState {
            phase: health.phase.label(),
            readiness: health.readiness.label(),
        })
    }
}
