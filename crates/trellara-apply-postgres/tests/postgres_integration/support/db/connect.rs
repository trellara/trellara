use super::*;

pub(crate) fn integration_database_url() -> Option<String> {
    match std::env::var(TEST_DATABASE_URL_ENV) {
        Ok(value) if !value.trim().is_empty() => Some(value),
        _ => None,
    }
}

pub(crate) async fn connect_test_client(database_url: &str) -> TestResult<tokio_postgres::Client> {
    let (client, connection) = tokio_postgres::connect(database_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            eprintln!("test postgres connection failed: {error}");
        }
    });
    Ok(client)
}

pub(crate) async fn connect_applier(database_url: &str) -> TestResult<PostgresApplier> {
    Ok(PostgresApplier::connect(PostgresApplyConfig {
        connection_uri: database_url.to_string(),
        ensure_checkpoint_schema: true,
        table_policies: Vec::new(),
    })
    .await?)
}

pub(crate) async fn connect_policy_applier(database_url: &str) -> TestResult<PostgresApplier> {
    Ok(PostgresApplier::connect(PostgresApplyConfig {
        connection_uri: database_url.to_string(),
        ensure_checkpoint_schema: true,
        table_policies: vec![ApplyTablePolicy {
            relation: relation(),
            target_owned_columns: vec!["reviewed_by".to_string()],
        }],
    })
    .await?)
}
