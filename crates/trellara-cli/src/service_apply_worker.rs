use trellara_apply_postgres::{
    ApplyStep, ApplyWorker, BarrierApplyStep, BarrierAwareApplyWorker, EnvelopeApplier,
};
use trellara_stream::StreamConsumer;

use crate::{ContinuousWorker, WorkerFuture};

pub(crate) enum ApplyServiceStep {
    Applied {
        step: ApplyStep,
        pending: trellara_apply_postgres::BarrierPendingStats,
    },
    Buffered {
        pending: trellara_apply_postgres::BarrierPendingStats,
    },
}

pub(crate) fn strict_worker<C, A>(
    consumer: C,
    applier: A,
) -> Box<dyn ContinuousWorker<Step = ApplyServiceStep>>
where
    C: StreamConsumer + 'static,
    A: EnvelopeApplier + 'static,
{
    Box::new(StrictServiceWorker {
        worker: ApplyWorker::new(consumer, applier),
    })
}

pub(crate) fn barrier_worker<C, A>(
    consumer: C,
    applier: A,
) -> Box<dyn ContinuousWorker<Step = ApplyServiceStep>>
where
    C: StreamConsumer + 'static,
    A: EnvelopeApplier + 'static,
{
    Box::new(BarrierServiceWorker {
        worker: BarrierAwareApplyWorker::new(consumer, applier),
        pending_work: 0,
    })
}

struct StrictServiceWorker<C, A> {
    worker: ApplyWorker<C, A>,
}

impl<C, A> ContinuousWorker for StrictServiceWorker<C, A>
where
    C: StreamConsumer,
    A: EnvelopeApplier,
{
    type Step = ApplyServiceStep;

    fn run_once(&mut self) -> WorkerFuture<'_, Self::Step> {
        Box::pin(async {
            self.worker
                .run_once()
                .await
                .map(|step| {
                    step.map(|step| ApplyServiceStep::Applied {
                        step,
                        pending: Default::default(),
                    })
                })
                .map_err(|_| "apply worker failed".to_string())
        })
    }
}

struct BarrierServiceWorker<C, A> {
    worker: BarrierAwareApplyWorker<C, A>,
    pending_work: u64,
}

impl<C, A> ContinuousWorker for BarrierServiceWorker<C, A>
where
    C: StreamConsumer,
    A: EnvelopeApplier,
{
    type Step = ApplyServiceStep;

    fn run_once(&mut self) -> WorkerFuture<'_, Self::Step> {
        Box::pin(async {
            let result = self
                .worker
                .run_once()
                .await
                .map_err(|_| "barrier apply worker failed".to_string())?;
            let pending = self.worker.pending_barrier_stats();
            self.pending_work = pending.transactions;
            Ok(result.map(|step| match step {
                BarrierApplyStep::Applied(step) => ApplyServiceStep::Applied { step, pending },
                BarrierApplyStep::Buffered { .. } => ApplyServiceStep::Buffered { pending },
            }))
        })
    }

    fn pending_work(&self) -> u64 {
        self.pending_work
    }
}
