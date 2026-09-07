use std::fs;

use rdkafka::config::ClientConfig;

use crate::{
    KafkaAuthenticationContract, KafkaProductionContract, KafkaSaslMechanism, KafkaSecretRef,
    KafkaStreamError,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KafkaSecurityConfig {
    ca_certificate: KafkaSecretRef,
    authentication: KafkaAuthenticationContract,
}

impl KafkaSecurityConfig {
    #[must_use]
    pub fn from_production_contract(contract: &KafkaProductionContract) -> Self {
        Self {
            ca_certificate: contract.ca_certificate.clone(),
            authentication: contract.authentication.clone(),
        }
    }

    pub(crate) fn apply(&self, config: &mut ClientConfig) -> Result<(), KafkaStreamError> {
        let ca = resolve_secret(&self.ca_certificate, "ca_certificate")?;
        config.set("ssl.ca.pem", ca);

        match &self.authentication {
            KafkaAuthenticationContract::MutualTls {
                client_certificate,
                client_key,
            } => {
                config
                    .set("security.protocol", "SSL")
                    .set(
                        "ssl.certificate.pem",
                        resolve_secret(client_certificate, "authentication.client_certificate")?,
                    )
                    .set(
                        "ssl.key.pem",
                        resolve_secret(client_key, "authentication.client_key")?,
                    );
            }
            KafkaAuthenticationContract::SaslTls {
                mechanism,
                username,
                password,
            } => {
                config
                    .set("security.protocol", "SASL_SSL")
                    .set("sasl.mechanism", sasl_mechanism(*mechanism))
                    .set(
                        "sasl.username",
                        resolve_secret(username, "authentication.username")?,
                    )
                    .set(
                        "sasl.password",
                        resolve_secret(password, "authentication.password")?,
                    );
            }
        }
        Ok(())
    }
}

fn sasl_mechanism(mechanism: KafkaSaslMechanism) -> &'static str {
    match mechanism {
        KafkaSaslMechanism::Plain => "PLAIN",
        KafkaSaslMechanism::ScramSha256 => "SCRAM-SHA-256",
        KafkaSaslMechanism::ScramSha512 => "SCRAM-SHA-512",
    }
}

fn resolve_secret(
    reference: &KafkaSecretRef,
    field: &'static str,
) -> Result<String, KafkaStreamError> {
    let value = match reference {
        KafkaSecretRef::EnvironmentVariable { name } => {
            std::env::var(name).map_err(|_| KafkaStreamError::SecretUnavailable {
                field,
                reason: "environment variable is unset or not valid Unicode",
            })?
        }
        KafkaSecretRef::File { path } => {
            fs::read_to_string(path).map_err(|_| KafkaStreamError::SecretUnavailable {
                field,
                reason: "secret file is unreadable or not valid UTF-8",
            })?
        }
    };
    let value = value
        .strip_suffix("\r\n")
        .or_else(|| value.strip_suffix('\n'))
        .unwrap_or(&value)
        .to_string();
    if value.is_empty() {
        Err(KafkaStreamError::SecretUnavailable {
            field,
            reason: "resolved secret is empty",
        })
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(name: &str) -> KafkaSecretRef {
        KafkaSecretRef::EnvironmentVariable {
            name: name.to_string(),
        }
    }

    #[test]
    fn sasl_tls_maps_to_librdkafka_without_inline_public_config() {
        let suffix = std::process::id();
        let ca_name = format!("TRELLARA_TEST_KAFKA_CA_{suffix}");
        let user_name = format!("TRELLARA_TEST_KAFKA_USER_{suffix}");
        let password_name = format!("TRELLARA_TEST_KAFKA_PASSWORD_{suffix}");
        std::env::set_var(&ca_name, "test-ca");
        std::env::set_var(&user_name, "alice");
        std::env::set_var(&password_name, "secret");
        let security = KafkaSecurityConfig {
            ca_certificate: env(&ca_name),
            authentication: KafkaAuthenticationContract::SaslTls {
                mechanism: KafkaSaslMechanism::ScramSha512,
                username: env(&user_name),
                password: env(&password_name),
            },
        };
        let mut config = ClientConfig::new();

        security.apply(&mut config).expect("security config");

        assert_eq!(config.get("security.protocol"), Some("SASL_SSL"));
        assert_eq!(config.get("sasl.mechanism"), Some("SCRAM-SHA-512"));
        assert_eq!(config.get("sasl.username"), Some("alice"));
        assert_eq!(config.get("sasl.password"), Some("secret"));
        std::env::remove_var(ca_name);
        std::env::remove_var(user_name);
        std::env::remove_var(password_name);
    }

    #[test]
    fn missing_secret_error_does_not_disclose_reference_name() {
        let reference_name = format!("TRELLARA_MISSING_KAFKA_SECRET_{}", std::process::id());
        std::env::remove_var(&reference_name);
        let error = resolve_secret(&env(&reference_name), "authentication.password")
            .expect_err("missing secret");

        assert!(!error.to_string().contains(&reference_name));
    }

    #[test]
    fn mutual_tls_maps_certificate_and_key_references() {
        let suffix = std::process::id();
        let ca_name = format!("TRELLARA_TEST_MTLS_CA_{suffix}");
        let certificate_name = format!("TRELLARA_TEST_MTLS_CERTIFICATE_{suffix}");
        let key_name = format!("TRELLARA_TEST_MTLS_KEY_{suffix}");
        std::env::set_var(&ca_name, "test-ca");
        std::env::set_var(&certificate_name, "test-certificate");
        std::env::set_var(&key_name, "test-key");
        let security = KafkaSecurityConfig {
            ca_certificate: env(&ca_name),
            authentication: KafkaAuthenticationContract::MutualTls {
                client_certificate: env(&certificate_name),
                client_key: env(&key_name),
            },
        };
        let mut config = ClientConfig::new();

        security.apply(&mut config).expect("mTLS config");

        assert_eq!(config.get("security.protocol"), Some("SSL"));
        assert_eq!(config.get("ssl.certificate.pem"), Some("test-certificate"));
        assert_eq!(config.get("ssl.key.pem"), Some("test-key"));
        std::env::remove_var(ca_name);
        std::env::remove_var(certificate_name);
        std::env::remove_var(key_name);
    }
}
