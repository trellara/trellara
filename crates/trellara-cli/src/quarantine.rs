use trellara_checkpoint::{FlowKey, TransactionKey};

use crate::{
    quarantine_replay_safety_contract_with_boundary, replay_redelivery_hint,
    replay_redelivery_plan, require_quarantine_replay_record, target_checkpoint_store, CliError,
    QuarantineClearSummary, QuarantineListSummary, QuarantineReplayReadySummary, Result,
    TrellaraConfig,
};

pub(crate) async fn list_quarantine(
    config: &TrellaraConfig,
    limit: i64,
) -> Result<QuarantineListSummary> {
    if limit <= 0 {
        return Err(CliError::InvalidConfig(
            "quarantine list --limit must be greater than zero".to_string(),
        ));
    }
    let store = target_checkpoint_store(config).await?;
    let flow = FlowKey::new(&config.source.id, &config.dataset.id);
    let records = store.list_quarantine(&flow, limit).await?;

    Ok(QuarantineListSummary {
        source_id: config.source.id.clone(),
        dataset_id: config.dataset.id.clone(),
        count: records.len(),
        records,
    })
}

pub(crate) async fn clear_quarantine(
    config: &TrellaraConfig,
    transaction_id: String,
    commit_lsn: String,
) -> Result<QuarantineClearSummary> {
    let transaction = quarantine_transaction_key(
        config,
        "quarantine.clear",
        transaction_id.clone(),
        commit_lsn.clone(),
    )?;
    let store = target_checkpoint_store(config).await?;
    let cleared = store.clear_quarantine(&transaction).await?;

    Ok(QuarantineClearSummary {
        source_id: config.source.id.clone(),
        dataset_id: config.dataset.id.clone(),
        transaction_id,
        commit_lsn,
        cleared,
    })
}

pub(crate) async fn prepare_quarantine_replay(
    config: &TrellaraConfig,
    transaction_id: String,
    commit_lsn: String,
) -> Result<QuarantineReplayReadySummary> {
    let transaction = quarantine_transaction_key(
        config,
        "quarantine.replay_ready",
        transaction_id.clone(),
        commit_lsn.clone(),
    )?;
    let store = target_checkpoint_store(config).await?;
    let quarantine =
        require_quarantine_replay_record(&transaction, store.load_quarantine(&transaction).await?)?;
    let dedup_removed = store.clear_applied_transaction(&transaction).await?;
    let quarantine_cleared = store.clear_quarantine(&transaction).await?;
    let redelivery_topics = config.replay_redelivery_topics()?;
    let redelivery_plan = replay_redelivery_plan(
        config,
        &redelivery_topics,
        Some((&transaction_id, &commit_lsn)),
    )?;
    let redelivery_hint = replay_redelivery_hint(config, &redelivery_topics);
    let safety_contract = quarantine_replay_safety_contract_with_boundary(
        dedup_removed,
        quarantine_cleared,
        redelivery_plan.boundary.as_ref(),
        redelivery_plan.exact_seek_commands.len(),
    );

    Ok(QuarantineReplayReadySummary {
        source_id: config.source.id.clone(),
        dataset_id: config.dataset.id.clone(),
        transaction_id,
        commit_lsn,
        quarantine_reason: quarantine.reason,
        quarantine_detail: quarantine.detail,
        dedup_removed,
        quarantine_cleared,
        redelivery_required: true,
        redelivery_topics,
        redelivery_boundary: redelivery_plan.boundary,
        redelivery_warnings: redelivery_plan.warnings,
        exact_seek_commands: redelivery_plan.exact_seek_commands,
        redelivery_commands: redelivery_plan.commands,
        redelivery_hint,
        safety_contract,
    })
}

pub(crate) fn quarantine_transaction_key(
    config: &TrellaraConfig,
    context: &'static str,
    transaction_id: String,
    commit_lsn: String,
) -> Result<TransactionKey> {
    if transaction_id.trim().is_empty() {
        return Err(CliError::InvalidConfig(format!(
            "{context}.transaction_id must not be empty"
        )));
    }
    if transaction_id.trim() != transaction_id {
        return Err(CliError::InvalidConfig(format!(
            "{context}.transaction_id must not contain surrounding whitespace"
        )));
    }
    if commit_lsn.trim().is_empty() {
        return Err(CliError::InvalidConfig(format!(
            "{context}.commit_lsn must not be empty"
        )));
    }
    if commit_lsn.trim() != commit_lsn {
        return Err(CliError::InvalidConfig(format!(
            "{context}.commit_lsn must not contain surrounding whitespace"
        )));
    }
    let transaction = TransactionKey {
        source_id: config.source.id.clone(),
        database_id: config
            .source
            .database_id
            .clone()
            .unwrap_or_else(|| config.dataset.id.clone()),
        dataset_id: config.dataset.id.clone(),
        transaction_id,
        commit_lsn,
    };
    transaction.validate()?;
    Ok(transaction)
}
