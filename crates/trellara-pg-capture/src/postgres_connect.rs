use rustls::RootCertStore;
use tokio_postgres::{Client, NoTls};
use tokio_postgres_rustls::MakeRustlsConnect;
use tracing::error;

use crate::{CaptureError, Result};

pub(crate) async fn connect_control_client(connection_uri: &str) -> Result<Client> {
    if postgres_url_option(connection_uri, "sslmode")
        .is_some_and(|value| value.eq_ignore_ascii_case("disable"))
    {
        let (client, connection) = tokio_postgres::connect(connection_uri, NoTls).await?;
        spawn_connection(connection);
        return Ok(client);
    }

    let tls = rustls_connector()?;
    let (client, connection) = tokio_postgres::connect(connection_uri, tls).await?;
    spawn_connection(connection);
    Ok(client)
}

fn rustls_connector() -> Result<MakeRustlsConnect> {
    let mut roots = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    let certs = rustls_native_certs::load_native_certs();
    let added_native_roots = roots.add_parsable_certificates(certs.certs).0;
    if roots.is_empty() {
        return Err(CaptureError::InvalidConfig(format!(
            "failed to configure TLS root certificates: {:?}",
            certs.errors
        )));
    }

    tracing::debug!(
        native_roots = added_native_roots,
        webpki_roots = webpki_roots::TLS_SERVER_ROOTS.len(),
        "configured PostgreSQL TLS root certificates"
    );
    let config = rustls::ClientConfig::builder_with_provider(
        rustls::crypto::ring::default_provider().into(),
    )
    .with_safe_default_protocol_versions()
    .map_err(|error| {
        CaptureError::InvalidConfig(format!("failed to configure TLS defaults: {error}"))
    })?
    .with_root_certificates(roots)
    .with_no_client_auth();

    Ok(MakeRustlsConnect::new(config))
}

fn spawn_connection<T>(connection: T)
where
    T: std::future::Future<Output = std::result::Result<(), tokio_postgres::Error>>
        + Send
        + 'static,
{
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            error!(%error, "postgres connection task failed");
        }
    });
}

fn postgres_url_option<'a>(database_url: &'a str, name: &str) -> Option<&'a str> {
    for token in database_url.split_whitespace() {
        if let Some(value) = token.strip_prefix(&format!("{name}=")) {
            return Some(value);
        }
    }

    let query = database_url.split_once('?')?.1;
    query.split('&').find_map(|part| {
        let (key, value) = part.split_once('=')?;
        key.eq_ignore_ascii_case(name).then_some(value)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn postgres_url_option_reads_uri_and_keyword_sslmode() {
        assert_eq!(
            postgres_url_option("postgresql://db/app?sslmode=require", "sslmode"),
            Some("require")
        );
        assert_eq!(
            postgres_url_option("host=db.example.com sslmode=disable user=app", "sslmode"),
            Some("disable")
        );
    }
}
