use crate::{ApplyWorkerError, ApplyWorkerResult};

pub(crate) fn checked_worker_stat_add(
    left: u64,
    right: u64,
    field: &'static str,
) -> ApplyWorkerResult<u64> {
    left.checked_add(right)
        .ok_or(ApplyWorkerError::StatOverflow { field })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_worker_stat_add_returns_sum() {
        assert_eq!(
            checked_worker_stat_add(4, 5, "counter").expect("checked add"),
            9
        );
    }

    #[test]
    fn checked_worker_stat_add_rejects_overflow() {
        let error = checked_worker_stat_add(u64::MAX, 1, "counter").expect_err("overflow");

        assert!(matches!(
            error,
            ApplyWorkerError::StatOverflow { field: "counter" }
        ));
    }
}
