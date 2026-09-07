use trellara_checkpoint::DdlBarrierStore;
use trellara_protocol::TransactionEnvelope;

use crate::{
    execute_target_ddl_transaction, plan_target_ddl_transaction,
    target_ddl_apply_plan_from_envelope, PostgresApplier, Result, TargetDdlAckContext,
    TargetDdlAckEvidence, TargetDdlApplyOutcome, TargetDdlApplyPlan,
};

impl PostgresApplier {
    pub async fn apply_target_ddl_plan(
        &mut self,
        plan: TargetDdlApplyPlan,
    ) -> Result<TargetDdlApplyOutcome> {
        let transaction_plan = plan_target_ddl_transaction(plan)?;
        let transaction = self.client.transaction().await?;
        let applied_statements =
            execute_target_ddl_transaction(&transaction, &transaction_plan).await?;
        transaction.commit().await?;

        Ok(TargetDdlApplyOutcome {
            barrier_id: transaction_plan.barrier_id,
            applied_statements,
            release_gate: transaction_plan.release_gate,
            plan_sha256: transaction_plan.plan_sha256,
            statement_sha256s: transaction_plan.statement_sha256s,
        })
    }

    pub async fn apply_target_ddl_plan_and_record_ack<S: DdlBarrierStore + ?Sized>(
        &mut self,
        plan: TargetDdlApplyPlan,
        store: &S,
        ack_context: TargetDdlAckContext,
    ) -> Result<TargetDdlAckEvidence> {
        let evidence = self
            .apply_target_ddl_plan(plan)
            .await?
            .target_ack_evidence_from_context(ack_context)?;
        store
            .record_ddl_barrier_ack(evidence.clone().into_barrier_ack())
            .await?;
        Ok(evidence)
    }

    pub async fn apply_target_ddl_from_envelope_and_record_ack<S: DdlBarrierStore + ?Sized>(
        &mut self,
        envelope: &TransactionEnvelope,
        store: &S,
        schema_version: impl Into<String>,
    ) -> Result<Option<TargetDdlAckEvidence>> {
        let Some(plan) = target_ddl_apply_plan_from_envelope(envelope)? else {
            return Ok(None);
        };
        self.apply_target_ddl_plan_and_record_ack(
            plan,
            store,
            TargetDdlAckContext::from_envelope(envelope, schema_version),
        )
        .await
        .map(Some)
    }
}
