use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use trellara_runtime::{RuntimeHealth, RuntimePhase, RuntimeReadiness, RuntimeService};

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ServiceSnapshot {
    pub(crate) health: RuntimeHealth,
    pub(crate) restarts_total: u64,
    pub(crate) failures_total: u64,
    pub(crate) reloads_total: u64,
    pub(crate) last_success_unixtime: u64,
    pub(crate) last_durable_lsn_bytes: u64,
}

#[derive(Clone)]
pub(crate) struct ServiceState(Arc<RwLock<ServiceSnapshot>>);

impl ServiceState {
    pub(crate) fn new(service: RuntimeService, source_id: &str, dataset_id: &str) -> Self {
        Self(Arc::new(RwLock::new(ServiceSnapshot {
            health: RuntimeHealth::starting(service, source_id, dataset_id),
            restarts_total: 0,
            failures_total: 0,
            reloads_total: 0,
            last_success_unixtime: 0,
            last_durable_lsn_bytes: 0,
        })))
    }

    pub(crate) fn snapshot(&self) -> ServiceSnapshot {
        self.0
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub(crate) fn mark_starting(&self, restarted: bool) {
        self.update(|snapshot| {
            snapshot.health.phase = RuntimePhase::Starting;
            snapshot.health.readiness = RuntimeReadiness::NotReady;
            snapshot.health.accepting_work = false;
            snapshot.health.pending_work = 0;
            snapshot.health.reason_code = None;
            if restarted {
                snapshot.restarts_total = snapshot.restarts_total.saturating_add(1);
            }
        });
    }

    pub(crate) fn mark_ready(&self) {
        self.update(|snapshot| {
            snapshot.health.phase = RuntimePhase::Running;
            snapshot.health.readiness = RuntimeReadiness::Ready;
            snapshot.health.accepting_work = true;
            snapshot.health.reason_code = None;
        });
    }

    pub(crate) fn mark_progress(&self, pending_work: u64, durable_lsn: Option<&str>) {
        self.update(|snapshot| {
            snapshot.health.pending_work = pending_work;
            if let Some(lsn) = durable_lsn {
                snapshot.last_success_unixtime = unix_time();
                if let Ok(bytes) = trellara_protocol::parse_lsn(lsn) {
                    snapshot.last_durable_lsn_bytes = bytes;
                }
            }
        });
    }

    pub(crate) fn mark_failure(&self, reason_code: &'static str) {
        self.update(|snapshot| {
            snapshot.health.phase = RuntimePhase::Running;
            snapshot.health.readiness = RuntimeReadiness::Blocked;
            snapshot.health.accepting_work = false;
            snapshot.health.pending_work = 0;
            snapshot.health.reason_code = Some(reason_code.to_string());
            snapshot.failures_total = snapshot.failures_total.saturating_add(1);
        });
    }

    pub(crate) fn mark_draining(&self, reload: bool) {
        self.update(|snapshot| {
            snapshot.health.phase = RuntimePhase::Draining;
            snapshot.health.readiness = RuntimeReadiness::NotReady;
            snapshot.health.accepting_work = false;
            snapshot.health.reason_code = None;
            if reload {
                snapshot.reloads_total = snapshot.reloads_total.saturating_add(1);
            }
        });
    }

    pub(crate) fn mark_stopped(&self) {
        self.update(|snapshot| {
            snapshot.health.phase = RuntimePhase::Stopped;
            snapshot.health.readiness = RuntimeReadiness::NotReady;
            snapshot.health.accepting_work = false;
            snapshot.health.pending_work = 0;
            snapshot.health.reason_code = None;
        });
    }

    fn update(&self, update: impl FnOnce(&mut ServiceSnapshot)) {
        let mut snapshot = self
            .0
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        update(&mut snapshot);
        debug_assert!(snapshot.health.validate().is_ok());
    }
}

fn unix_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}
